use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, ThemeMode, TitleBar, WindowExt,
    avatar::Avatar,
    button::{Button, ButtonVariants},
    menu::DropdownMenu,
    notification::Notification,
    v_flex,
};
use uuid::Uuid;

use crate::{
    auth::AuthState,
    ctx::UserState,
    nav::{NavEvent, NavState, Navigation},
    rt,
    theme::*,
    ui::{
        _components::{app_icon, create_space_dialog, select_space_dialog},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api,
};

actions!([EmptyAction]);

pub struct HeaderUi {
    auth: AuthState,
    user_state: UserState,
    nav: Navigation,

    loading_spaces: bool,
    loading_user: bool,
    creating_space: bool,

    _subscriptions: Vec<Subscription>,
}

impl HeaderUi {
    pub fn new(
        auth: AuthState,
        user_state: UserState,
        nav: Navigation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                crate::auth::AuthEvent::Session(session_state) => match session_state {
                    crate::auth::SessionState::SignedIn => {
                        this.fetch_user(window, cx);
                        this.fetch_spaces(None, window, cx);
                    }
                    _ => (),
                },
                _ => (),
            };
        });

        let nav_sub = cx.subscribe_in(&nav, window, |this, _entity, event, window, cx| {
            match event {
                NavEvent::Refresh => {
                    this.fetch_user(window, cx);
                    this.fetch_spaces(None, window, cx);
                }
                NavEvent::NewSpace(space_id) => {
                    this.fetch_spaces(Some(space_id.clone()), window, cx);
                }
            };
        });

        Self {
            auth,
            user_state,
            nav,
            loading_spaces: false,
            loading_user: false,
            creating_space: false,
            _subscriptions: vec![auth_sub, nav_sub],
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

    fn change_mode(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if cx.theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        change_color_mode(new_mode, cx);
    }

    fn fetch_user(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();

        let user_task = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| api::user::get_user(&token).await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_user = true;
                cx.notify();
            });

            let result = user_task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_user = false;
                match result {
                    Ok(user) => this.user_state.update(cx, |ctx, _cx| ctx.user = Some(user)),
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to fetch user"),
                        cx,
                    ),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn fetch_spaces(
        &self,
        set_active_space: Option<Uuid>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inner = self.auth.read(cx).inner();

        let user_spaces = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| api::space::get_user_spaces(token).await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_spaces = true;
                cx.notify();
            });

            let user_spaces = user_spaces.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_spaces = false;
                match user_spaces {
                    Ok(user_spaces) => {
                        window.close_all_dialogs(cx);

                        if let Some(user_space) = set_active_space.and_then(|space_id| {
                            user_spaces
                                .iter()
                                .find(|us| us.space.id == space_id)
                                .cloned()
                        }) {
                            this.nav.update(cx, |stack, cx| {
                                stack.push(
                                    BrowseUi::view(
                                        this.auth.clone(),
                                        this.user_state.clone(),
                                        this.nav.clone(),
                                        NavState::new(user_space.space.id, user_space.folder),
                                        window,
                                        cx,
                                    ),
                                    cx,
                                );
                            });
                        }

                        this.user_state
                            .update(cx, |ctx, _cx| ctx.user_spaces = user_spaces);
                    }

                    Err(err) => window.push_notification(
                        Notification::error(err.message)
                            .title("Failed to load spaces")
                            .autohide(false),
                        cx,
                    ),
                };
                cx.notify();
            });
        })
        .detach();
    }
}

impl create_space_dialog::CreateSpaceDialog for HeaderUi {
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
                        this.fetch_spaces(Some(space.id), window, cx);
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

impl Render for HeaderUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_toggle = Button::new("theme-mode")
            .icon(Icon::empty().path("icons/circle-shade.svg"))
            .small()
            .ghost()
            .on_click(cx.listener(Self::change_mode));

        TitleBar::new()
            .absolute()
            .top_0()
            .w_full()
            .child(self.render_nav_buttons(cx))
            .child(self.render_space_switcher(cx))
            .child(
                div()
                    .pr(px(5.0))
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(self.render_user_popup(cx))
                    .child(theme_toggle),
            )
    }
}

impl HeaderUi {
    fn render_user_popup(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.weak_entity();

        Button::new("user")
            .loading(self.loading_user)
            .loading_icon(Icon::new(IconName::LoaderCircle))
            .small()
            .size_6()
            .ghost()
            .map(|this| match self.user_state.read(cx).user.as_ref() {
                Some(user) => {
                    let user = user.clone();

                    this.child(
                        Avatar::new()
                            .with_size(gpui_component::Size::Small)
                            .name(&user.given_name)
                            .src(user.picture_url.clone())
                            .placeholder(Icon::new(IconName::CircleUser))
                            .size_4(),
                    )
                    .dropdown_menu(move |menu, _window, cx| {
                        let user = user.clone();
                        let entity = entity.clone();

                        menu.menu_element(Box::new(EmptyAction), move |_window, cx| {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Avatar::new()
                                        .name(&user.given_name)
                                        .src(user.picture_url.clone())
                                        .placeholder(Icon::new(IconName::CircleUser))
                                        .size_8(),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0()
                                        .child(div().child(user.given_name.clone()).text_sm())
                                        .child(
                                            div()
                                                .child(user.email.clone())
                                                .text_color(cx.theme().muted_foreground)
                                                .text_sm(),
                                        ),
                                )
                        })
                        .separator()
                        .menu_element(
                            Box::new(EmptyAction),
                            move |_window, _cx| {
                                let entity = entity.clone();

                                div()
                                    .id("logout")
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::empty().path("icons/log-out.svg"))
                                    .child(div().child("Logout").text_sm())
                                    .on_click(move |_ev, window, cx| {
                                        cx.stop_propagation();

                                        let _ = entity.update(cx, |this, cx| {
                                            cx.notify();
                                        });
                                    })
                            },
                        )
                    })
                }
                None => this.dropdown_menu(|m, _, _| m),
            })
    }

    fn render_space_switcher(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.weak_entity();

        Button::new("space_switcher")
            .small()
            .compact()
            .icon(Icon::new(IconName::GalleryVerticalEnd))
            .dropdown_caret(true)
            .map(|this| {
                let us = self.nav.read(cx).current_space_id().and_then(|sp_id| {
                    self.user_state
                        .read(cx)
                        .user_spaces
                        .iter()
                        .find_map(|us| {
                            if &us.space.id == sp_id {
                                Some(us)
                            } else {
                                None
                            }
                        })
                        .cloned()
                });

                match us {
                    Some(us) => this.child(
                        v_flex()
                            .items_center()
                            .justify_center()
                            .w_40()
                            .overflow_hidden()
                            .truncate()
                            .text_ellipsis()
                            .child(
                                div()
                                    .whitespace_normal()
                                    .child(us.space.name.clone())
                                    .text_sm(),
                            ),
                    ),
                    None => this.child(
                        v_flex()
                            .items_center()
                            .justify_center()
                            .w_40()
                            .child("Select space"),
                    ),
                }
            })
            .text_sm()
            .disabled(self.loading_spaces)
            .loading(self.loading_spaces)
            .loading_icon(IconName::LoaderCircle)
            .on_click(move |_ev, window, cx| {
                cx.stop_propagation();
                let entity = entity.clone();

                window.open_dialog(cx, move |dialog, _window, cx| {
                    let entity = entity.clone();
                    let user_spaces = entity
                        .read_with(cx, |this, cx| this.user_state.read(cx).user_spaces.clone())
                        .unwrap_or_default();

                    select_space_dialog::comp::<Self>(
                        dialog,
                        entity,
                        user_spaces,
                        cx,
                        |entity, state, window, cx| {
                            entity
                                .clone()
                                .update(cx, |this, cx| {
                                    this.nav.update(cx, |stack, cx| {
                                        if let Some(current_space_id) = stack.current_space_id()
                                            && current_space_id == &state.space_id
                                        {
                                            // skip
                                            return;
                                        }

                                        stack.push(
                                            BrowseUi::view(
                                                this.auth.clone(),
                                                this.user_state.clone(),
                                                this.nav.clone(),
                                                state,
                                                window,
                                                cx,
                                            ),
                                            cx,
                                        );
                                    });
                                })
                                .ok();

                            window.close_dialog(cx);
                        },
                    )
                });
            })
    }

    fn render_nav_buttons(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_0p5()
            .child(
                Button::new("go-back")
                    .icon(Icon::new(IconName::ArrowLeft))
                    .small()
                    .ghost()
                    .disabled(self.nav.read(cx).at_begining())
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        cx.stop_propagation();

                        this.nav.update(cx, |stack, cx| {
                            stack.back(cx);
                        });
                    })),
            )
            .child(
                Button::new("go-forward")
                    .icon(Icon::new(IconName::ArrowRight))
                    .small()
                    .ghost()
                    .disabled(self.nav.read(cx).at_end())
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        cx.stop_propagation();

                        this.nav.update(cx, |stack, cx| {
                            stack.forward(cx);
                        });
                    })),
            )
            .child(
                Button::new("refresh")
                    .icon(Icon::empty().path("icons/rotate-ccw.svg"))
                    .small()
                    .ghost()
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        cx.stop_propagation();

                        this.nav.update(cx, |_nav, cx| {
                            cx.emit(NavEvent::Refresh);
                        });
                    })),
            )
            .child(
                app_icon::comp(cx, |icon| icon.size_4())
                    .p_1()
                    .id("home")
                    .ml_1()
                    .hover(|el| el.bg(cx.theme().primary_hover))
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        cx.stop_propagation();

                        this.nav.update(cx, |stack, cx| {
                            stack.home(cx);
                        })
                    })),
            )
    }
}
