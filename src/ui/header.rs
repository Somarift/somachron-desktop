use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme as _, ContextModal, Icon, IconName, Sizable, StyledExt, ThemeMode,
    avatar::Avatar,
    button::{Button, ButtonVariants as _, DropdownButton},
    label::Label,
    notification::Notification,
};

use crate::{
    api::{self, models::user::res::UserResponse},
    auth::{Auth, AuthEvent, SessionState},
    theme::*,
};

const TITLE_BAR_LEFT_PADDING: Pixels = px(80.);

actions!(user, [MyAction, SignOut]);

pub struct HeaderUi {
    auth: Entity<Auth>,
    user: Option<UserResponse>,
    _subscription: Vec<Subscription>,
}

impl HeaderUi {
    pub fn new(auth: Entity<Auth>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let session_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                AuthEvent::Session(session_state) => match session_state {
                    SessionState::SignedIn => {
                        this.fetch_user(window, cx);
                    }
                    _ => (),
                },
                _ => (),
            };
        });

        Self {
            auth,
            user: None,
            _subscription: vec![session_sub],
        }
    }

    pub fn view(auth: Entity<Auth>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, window, cx))
    }

    pub fn change_mode(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if cx.theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        change_color_mode(new_mode, cx);
    }

    fn fetch_user(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .read_with(cx, |this, cx| this.auth.read(cx).inner())
                .unwrap();

            let result = cx
                .background_executor()
                .spawn(async move { inner.get_token().await })
                .await;

            let user = match result {
                Ok(token) => {
                    cx.background_executor()
                        .spawn(async move { api::user::get_user(&token).await })
                        .await
                }
                Err(err) => Err(err),
            };

            this.update_in(cx, |this, window, cx| {
                match user {
                    Ok(user) => {
                        this.user = Some(user);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(&err.message)
                                .title("Failed to fetch user")
                                .autohide(true),
                            cx,
                        );
                    }
                };
                cx.notify();
            })
            .unwrap();
        })
        .detach();
    }

    fn sign_out(&mut self, cx: &mut Context<Self>) {}
}

impl Render for HeaderUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_toggle = Button::new("theme-mode")
            .icon(Icon::empty().path("icons/circle-shade.svg"))
            .small()
            .ghost()
            .on_click(cx.listener(Self::change_mode));

        let user_button = Button::new("user")
            .when_none(&self.user, |el| el.icon(IconName::CircleUser))
            .when_some(self.user.clone(), |el, user| {
                el.child(
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .justify_center()
                        .child(
                            Avatar::new()
                                .name(&user.given_name)
                                .src(user.picture_url)
                                .small(),
                        )
                        .child(div().text_xs().child(user.given_name)),
                )
            });

        div()
            .id("header-bar")
            .on_action(|a: &SignOut, b, c| {
                // self.sign_out(cx);
            })
            .border_b_1()
            .bg(cx.theme().title_bar)
            .border_color(cx.theme().border)
            .pl(TITLE_BAR_LEFT_PADDING)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_1()
                    .child(Label::new("Somachron").text_xs().font_medium())
                    .child(
                        div()
                            .pr(px(5.0))
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(theme_toggle)
                            .child(
                                DropdownButton::new("user-menu")
                                    .button(user_button)
                                    .compact()
                                    .small()
                                    .small()
                                    .text_sm()
                                    .popup_menu(|menu, _, _| {
                                        menu.menu("Option 1", Box::new(MyAction))
                                            .menu("Option 2", Box::new(MyAction))
                                            .separator()
                                            .menu_with_icon(
                                                "Sign out",
                                                Icon::empty().path("icons/logout.svg"),
                                                Box::new(SignOut),
                                            )
                                    }),
                            ),
                    ),
            )
    }
}
