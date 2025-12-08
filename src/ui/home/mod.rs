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
};

use crate::{
    auth::Auth,
    rt,
    ui::{
        _components::{self, NavContext, NavStack, NavState, RenderBounds, loading_icon},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api::{
        self,
        models::{space::res::UserSpaceResponse, user::res::UserResponse},
    },
};

mod browse;

actions!(user, [MyAction, SignOut]);

pub struct HomeUi {
    auth: Entity<Auth>,
    nav_stack: Entity<NavStack>,
    nav_ctx: NavContext<NavStack>,
    render_bounds: Entity<RenderBounds>,

    user_spaces: Vec<UserSpaceResponse>,
    user: Option<UserResponse>,
    loading_sidebar: bool,
    loading_user: bool,
    _subscriptions: Vec<Subscription>,
}

impl HomeUi {
    fn new(auth: Entity<Auth>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let scroll_bounds = cx.new(|_cx| RenderBounds::new());
        let nav_stack = cx.new(|_cx| NavStack::new());
        let nav_ctx = NavContext::new(nav_stack.downgrade());

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                crate::auth::AuthEvent::Session(session_state) => match session_state {
                    crate::auth::SessionState::SignedIn => {
                        this.fetch_data(window, cx);
                    }
                    _ => (),
                },
                _ => (),
            };
        });

        Self {
            auth,
            nav_stack,
            nav_ctx,
            render_bounds: scroll_bounds,
            user_spaces: Vec::new(),
            user: None,
            loading_sidebar: false,
            loading_user: false,
            _subscriptions: vec![auth_sub],
        }
    }

    pub fn view(auth: Entity<Auth>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, window, cx))
    }

    fn fetch_data(&self, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();

        let _inner = inner.clone();
        let user_spaces = rt::spawn(cx, async move {
            _inner
                .get_token()
                .await
                .map_async(async move |token| api::space::get_user_spaces(token).await)
                .await
        });

        let user = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| api::user::get_user(&token).await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_user = true;
                this.loading_sidebar = true;
                cx.notify();
            });

            let user_spaces = user_spaces.await.flatten();
            let user = user.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_sidebar = false;
                match user_spaces {
                    Ok(user_spaces) => {
                        this.user_spaces = user_spaces;
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(err.message)
                                .title("Failed to load spaces")
                                .autohide(false),
                            cx,
                        );
                    }
                };

                this.loading_user = false;
                match user {
                    Ok(user) => {
                        this.user = Some(user);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(&err.message)
                                .title("Failed to fetch user")
                                .autohide(false),
                            cx,
                        );
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for HomeUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(
                div().border_1().border_color(cx.theme().border), // .child(self.render_sidebar(cx)),
            )
            .map(|el| {
                let current = self.nav_stack.read(cx).current().cloned();
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
            } else if self.user_spaces.is_empty() {
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
                this.child(Label::new("Spaces"))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .children(self.user_spaces.iter().map(|m| {
                                let space_name = m.space.name.clone();
                                let space_description = if m.space.description.is_empty() {
                                    String::from("No description")
                                } else {
                                    m.space.description.clone()
                                };

                                let space_id = m.space.id.clone();
                                let folder_id = m.folder.clone();

                                div()
                                    .id(SharedString::new(m.space.id.clone()))
                                    .flex()
                                    .gap_4()
                                    .border_1()
                                    .rounded_md()
                                    .items_center()
                                    .p_4()
                                    .bg(cx.theme().sidebar)
                                    .w_56()
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
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .child(m.space.name.clone()),
                                            )
                                            .child(
                                                div()
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
                                        this.nav_ctx.update(cx, |ctx, cx| {
                                            ctx.clear_and_push(
                                                BrowseUi::view(
                                                    this.auth.clone(),
                                                    this.nav_ctx.clone(),
                                                    this.render_bounds.clone(),
                                                    NavState::new(
                                                        space_id.clone(),
                                                        folder_id.clone(),
                                                    ),
                                                    window,
                                                    cx,
                                                ),
                                                cx,
                                            );
                                        });
                                    }))
                            })),
                    )
            }
        })
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        Sidebar::new(Side::Left)
            .border_width(0.)
            .header(SidebarHeader::new().when_else(
                self.loading_user,
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
                    el.when_none(&self.user, |el| {
                        el.child(h_flex().gap_2().child("No user :/"))
                    })
                    .when_some(self.user.clone(), |el, user| {
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
                    })
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
                            self.user_spaces.is_empty(),
                            |el| el.child(SidebarMenuItem::new("No spaces")),
                            |el| {
                                el.children(self.user_spaces.iter().cloned().map(|us| {
                                    SidebarMenuItem::new(&us.space.name)
                                        .icon(Icon::new(IconName::GalleryVerticalEnd))
                                        .on_click(cx.listener(move |this, _ev, window, cx| {
                                            this.nav_ctx.update(cx, |ctx, cx| {
                                                ctx.clear_and_push(
                                                    BrowseUi::view(
                                                        this.auth.clone(),
                                                        this.nav_ctx.clone(),
                                                        this.render_bounds.clone(),
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
                                        }))
                                }))
                            },
                        )
                    },
                )),
            )
    }
}
