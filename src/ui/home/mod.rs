use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Side, StyledExt, WindowExt,
    avatar::Avatar,
    button::Button,
    h_flex,
    label::Label,
    notification::Notification,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    tooltip::Tooltip,
    v_flex,
};

use crate::{
    auth::AuthState,
    ctx::UserState,
    nav::{NavEvent, NavState, Navigation},
    rt,
    ui::{
        _components::{self, RenderBounds, create_space_dialog, loading_icon},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api,
};

pub(super) mod browse;

actions!(user, [MyAction, SignOut]);

pub struct HomeUi {
    auth: AuthState,
    user_state: UserState,
    nav: Navigation,
    render_bounds: Entity<RenderBounds>,

    loading_sidebar: bool,
    creating_space: bool,
    _subscriptions: Vec<Subscription>,
}

impl HomeUi {
    fn new(
        auth: AuthState,
        user_state: UserState,
        nav: Navigation,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let scroll_bounds = cx.new(|_cx| RenderBounds::new());

        Self {
            auth,
            user_state,
            nav,
            render_bounds: scroll_bounds,
            loading_sidebar: false,
            creating_space: false,
            _subscriptions: vec![],
        }
    }

    pub fn view(
        auth: AuthState,
        user_state: UserState,
        nav: Navigation,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, user_state, nav, window, cx))
    }
}

impl create_space_dialog::CreateSpaceDialog for HomeUi {
    fn create_space(
        &mut self,
        name: SharedString,
        description: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inner = self.auth.read(cx).inner();

        let task = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::space::create_space(
                        token,
                        name.as_str().to_owned(),
                        description.as_str().to_owned(),
                    )
                    .await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.creating_space = true;
                cx.notify();
            });

            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(space) => {
                        this.nav.update(cx, |_nav, cx| {
                            cx.emit(NavEvent::NewSpace(space.id));
                        });
                    }
                    Err(err) => {
                        this.creating_space = false;

                        window.push_notification(
                            Notification::error(err.message).title("Failed to create space"),
                            cx,
                        );
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn is_loading(&self) -> bool {
        self.creating_space
    }
}

impl Render for HomeUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            // .child(
            //     div().border_1().border_color(cx.theme().border), // .child(self.render_sidebar(cx)),
            // )
            .map(|el| {
                let current = self.nav.read(cx).current().cloned();
                match current {
                    Some(view) => el.child(div().size_full().child(view)),
                    None => el.child(self.render_spaces(cx)),
                }
            })
            .child({
                let this = cx.entity();
                canvas(
                    move |_, _, _| {},
                    move |el_bounds, _d, _w, cx| {
                        this.update(cx, |this, cx| {
                            this.render_bounds.update(cx, |bounds, cx| {
                                bounds.h_event(el_bounds.size.height).map(|ev| cx.emit(ev));
                            })
                        });
                    },
                )
            })
    }
}

impl HomeUi {
    fn render_spaces(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().w_full().gap_2().p_4().map(|this| {
            if self.loading_sidebar {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .items_center()
                        .justify_center()
                        .border_color(cx.theme().sidebar_border)
                        .border_1()
                        .border_dashed()
                        .rounded_lg()
                        .p_4()
                        .gap_2()
                        .child(loading_icon(|icon| icon.size_5())),
                )
            } else if self.user_state.read(cx).user_spaces.is_empty() {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .items_center()
                        .justify_center()
                        .border_color(cx.theme().sidebar_border)
                        .border_1()
                        .border_dashed()
                        .rounded_lg()
                        .p_4()
                        .gap_2()
                        .child(
                            div()
                                .rounded_md()
                                .p_2()
                                .bg(cx.theme().muted)
                                .child(Icon::new(IconName::GalleryVerticalEnd).size_5()),
                        )
                        .child(div().text_lg().child("Cloud storage empty"))
                        .child(
                            div().child(
                                "Create your space and upload files to access them anywhere.",
                            ),
                        )
                        .child(Button::new("create_new_space").label("Create space")),
                )
            } else {
                this.child(
                    h_flex()
                        .w_full()
                        .items_center()
                        .justify_between()
                        .child(Label::new("Spaces"))
                        .child(create_space_dialog::trigger(
                            cx.weak_entity(),
                            gpui_component::Size::Small,
                        )),
                )
                .child(div().flex().flex_wrap().gap_2().children(
                    self.user_state.read(cx).user_spaces.iter().map(|m| {
                        let space_name = m.space.name.clone();
                        let space_description = if m.space.description.is_empty() {
                            String::from("No description")
                        } else {
                            m.space.description.clone()
                        };

                        let space_id = m.space.id.clone();
                        let folder_id = m.folder.clone();

                        div()
                            .id(SharedString::new(m.space.id.to_string()))
                            .flex()
                            .gap_4()
                            .border_1()
                            .rounded_md()
                            .items_center()
                            .p_4()
                            .bg(cx.theme().sidebar)
                            .w_56()
                            .hover(|el| el.bg(cx.theme().secondary_hover))
                            .child(Icon::new(IconName::GalleryVerticalEnd).size_4())
                            .tooltip(move |window, cx| {
                                let space_name = space_name.clone();
                                let space_description = space_description.clone();
                                Tooltip::element(move |_, cx| {
                                    div().child(Label::new(space_name.clone())).child(
                                        Label::new(space_description.clone())
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                })
                                .build(window, cx)
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_wrap()
                                    .truncate()
                                    .text_ellipsis()
                                    .child(
                                        v_flex()
                                            .whitespace_normal()
                                            .text_sm()
                                            .font_medium()
                                            .child(m.space.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .whitespace_normal()
                                            .flex()
                                            .flex_wrap()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(if m.space.description.is_empty() {
                                                String::from("No description")
                                            } else {
                                                m.space.description.clone()
                                            }),
                                    ),
                            )
                            .on_click(cx.listener(move |this, _ev, window, cx| {
                                this.nav.update(cx, |stack, cx| {
                                    stack.push(
                                        BrowseUi::view(
                                            this.auth.clone(),
                                            this.user_state.clone(),
                                            this.nav.clone(),
                                            NavState::new(space_id.clone(), folder_id.clone()),
                                            window,
                                            cx,
                                        ),
                                        cx,
                                    );
                                });
                            }))
                    }),
                ))
            }
        })
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        Sidebar::new(Side::Left)
            .border_width(0.)
            .header(SidebarHeader::new().when_else(
                true,
                |el| {
                    el.child(
                        h_flex()
                            .gap_2()
                            .text_color(cx.theme().muted_foreground)
                            .child(_components::loading_icon(|icon| icon.size_6()))
                            .child("Loading"),
                    )
                },
                |el| {
                    el.when_none(&self.user_state.read(cx).user, |el| {
                        el.child(h_flex().gap_2().child("No user :/"))
                    })
                    .when_some(
                        self.user_state.read(cx).user.clone(),
                        |el, user| {
                            el.child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .child(
                                                Avatar::new()
                                                    .name(&user.given_name)
                                                    .src(user.picture_url)
                                                    .size_8(),
                                            )
                                            .child(user.given_name),
                                    )
                                    .child(Icon::new(IconName::ChevronsUpDown).size_4()),
                            )
                        },
                    )
                },
            ))
            .child(
                SidebarGroup::new("Spaces").child(SidebarMenu::new().when_else(
                    self.loading_sidebar,
                    |el| {
                        el.child(
                            SidebarMenuItem::new("Loading")
                                .active(false)
                                .suffix(_components::loading_icon(|icon| icon.size_4())),
                        )
                    },
                    |el| {
                        el.when_else(
                            self.user_state.read(cx).user_spaces.is_empty(),
                            |el| el.child(SidebarMenuItem::new("No spaces")),
                            |el| {
                                el.children(
                                    self.user_state.read(cx).user_spaces.iter().cloned().map(
                                        |us| {
                                            SidebarMenuItem::new(&us.space.name)
                                                .icon(Icon::new(IconName::GalleryVerticalEnd))
                                                .on_click(cx.listener(
                                                    move |this, _ev, window, cx| {
                                                        this.nav.update(cx, |stack, cx| {
                                                            stack.push(
                                                                BrowseUi::view(
                                                                    this.auth.clone(),
                                                                    this.user_state.clone(),
                                                                    this.nav.clone(),
                                                                    NavState::new(
                                                                        us.space.id.clone(),
                                                                        us.folder.clone(),
                                                                    ),
                                                                    window,
                                                                    cx,
                                                                ),
                                                                cx,
                                                            );
                                                        });
                                                    },
                                                ))
                                        },
                                    ),
                                )
                            },
                        )
                    },
                )),
            )
    }
}
