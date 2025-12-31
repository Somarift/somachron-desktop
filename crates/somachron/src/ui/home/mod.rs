use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt, WindowExt, h_flex, label::Label,
    notification::Notification, tooltip::Tooltip, v_flex,
};

use crate::{
    auth::AuthState,
    entities::{
        UserData,
        nav::{NavEvent, NavState, Navigation},
        transfer::TransferManager,
    },
    rt,
    ui::{
        _components::{create_space_dialog, loading_icon},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api,
};

pub(super) mod browse;
mod media;

actions!(user, [MyAction, SignOut]);

pub struct HomeUi {
    auth: AuthState,
    user_data: Entity<UserData>,
    transfer_manager: Entity<TransferManager>,
    nav: Navigation,

    loading_sidebar: bool,
    creating_space: bool,
    _subscriptions: Vec<Subscription>,
}

impl HomeUi {
    fn new(
        auth: AuthState,
        user_data: Entity<UserData>,
        transfer_manager: Entity<TransferManager>,
        nav: Navigation,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            auth,
            user_data,
            transfer_manager,
            nav,
            loading_sidebar: false,
            creating_space: false,
            _subscriptions: vec![],
        }
    }

    pub fn view(
        auth: AuthState,
        user_data: Entity<UserData>,
        transfer_manager: Entity<TransferManager>,
        nav: Navigation,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, user_data, transfer_manager, nav, window, cx))
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
            .pt(px(34.))
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
        // .child({
        //     let this = cx.entity();
        //     canvas(
        //         move |_, _, _| {},
        //         move |el_bounds, _d, _w, cx| {
        //             this.update(cx, |this, cx| {
        //                 this.render_bounds.update(cx, |bounds, cx| {
        //                     bounds.h_event(el_bounds.size.height).map(|ev| cx.emit(ev));
        //                 })
        //             });
        //         },
        //     )
        // })
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
            } else if self.user_data.read(cx).user_spaces.is_empty() {
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
                        .child(create_space_dialog::trigger(cx.weak_entity())),
                )
            } else {
                this.child(
                    h_flex()
                        .w_full()
                        .items_center()
                        .justify_between()
                        .child(Label::new("Spaces"))
                        .child(create_space_dialog::trigger(cx.weak_entity()).small()),
                )
                .child(div().flex().flex_wrap().gap_2().children(
                    self.user_data.read(cx).user_spaces.iter().map(|m| {
                        let space_name = m.space.name.clone();
                        let space_description = if m.space.description.is_empty() {
                            String::from("No description")
                        } else {
                            m.space.description.clone()
                        };

                        let space_id = m.space.id;
                        let folder_id = m.folder;

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
                                            this.user_data.clone(),
                                            this.nav.clone(),
                                            this.transfer_manager.clone(),
                                            NavState::new(space_id, folder_id),
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
}
