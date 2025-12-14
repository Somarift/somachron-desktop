use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    input::{Input, InputState, OtpInput, OtpState},
    notification::Notification,
};

use crate::{
    auth::{AuthEvent, AuthState, SessionState},
    rt,
    ui::_components::app_icon,
    util::MapAsync,
    web::api,
};

pub struct LoginUi {
    auth: AuthState,
    email_input: Entity<InputState>,
    otp_input: Entity<OtpState>,
    loading: bool,
    otp_verification: bool,
    _subscriptions: Vec<Subscription>,
}

impl LoginUi {
    fn new(auth: AuthState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let email_input = cx.new(|cx| InputState::new(window, cx).placeholder("user@email.com"));
        let otp_input = cx.new(|cx| OtpState::new(6, window, cx));

        let otp_sub = cx.subscribe_in(
            &otp_input,
            window,
            |this, _, event, window, cx| match event {
                gpui_component::input::InputEvent::Change => {
                    this.verify_otp(window, cx);
                }
                _ => (),
            },
        );

        Self {
            auth,
            email_input,
            otp_input,
            loading: false,
            otp_verification: false,
            _subscriptions: vec![otp_sub],
        }
    }

    pub fn view(auth: AuthState, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, window, cx))
    }

    fn sign_in(&self, window: &mut Window, cx: &mut Context<Self>) {
        let email = self.email_input.read(cx).value();

        let inner = self.auth.read(cx).inner();
        let sign_in_task = rt::spawn(cx, async move {
            inner
                .sign_in(&email)
                .await
                .map_async(async move |idn| inner.prepare_first_factor(&idn).await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            this.update(cx, |this, cx| {
                this.loading = true;
                cx.notify();
            })
            .unwrap();

            let result = sign_in_task.await.flatten();

            this.update_in(cx, |this, window, cx| {
                this.loading = false;
                match result {
                    Ok(_) => {
                        this.otp_verification = true;
                        this.otp_input.focus_handle(cx);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(err.message)
                                .title("Failed to login")
                                .autohide(false),
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

    fn verify_otp(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let code = self.otp_input.read(cx).value();
        if code.len() != 6 {
            return;
        }

        let code = code.clone();
        self.loading = true;

        let inner = self.auth.read(cx).inner();
        let otp_task = rt::spawn(cx, async move {
            inner
                .attempt_first_factor(&code)
                .await
                .map_async(async move |_| inner.fetch_token().await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = otp_task.await.flatten();

            this.update_in(cx, |this, window, cx| {
                this.loading = false;
                match result {
                    Ok(_) => {
                        this.reset_otp_verification(None, cx);

                        this.auth.update(cx, |_, cx| {
                            cx.emit(AuthEvent::Session(SessionState::SignedIn));
                        });
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(err.message)
                                .title("OTP verification failed")
                                .autohide(false),
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

    fn reset_otp_verification(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        self.loading = false;
        self.otp_input.update(cx, |otp, cx| {
            if let Some(window) = window {
                otp.set_value("", window, cx);
            }
            cx.notify();
        });
        self.otp_verification = false;
        self.email_input.update(cx, |state, cx| {
            state.focus_handle(cx);
            cx.notify();
        });
    }
}

impl Render for LoginUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_6()
                .justify_center()
                .items_center()
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(app_icon::comp(cx, |icon| icon.size_5()))
                        .child(div().font_medium().child("Somachron")),
                )
                .child(
                    div()
                        .max_w_96()
                        .w_full()
                        .flex()
                        .flex_col()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_xl()
                        .p_6()
                        .gap_6()
                        .bg(cx.theme().accent)
                        .child(
                            div()
                                .text_center()
                                .child(div().text_lg().font_semibold().child("Welcome back"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Login with your email"),
                                ),
                        )
                        .child(
                            div()
                                .grid()
                                .gap_4()
                                .child(
                                    div()
                                        .grid()
                                        .gap_2()
                                        .child(div().text_sm().font_medium().child("Email"))
                                        .child(
                                            Input::new(&self.email_input)
                                                .text_sm()
                                                .cleanable(true)
                                                .disabled(self.otp_verification)
                                                .line_clamp(1),
                                        ),
                                )
                                .when(self.otp_verification, |d| {
                                    d.child(
                                        OtpInput::new(&self.otp_input)
                                            .groups(2)
                                            .disabled(self.loading)
                                            .large(),
                                    )
                                    .child(
                                        Button::new("login-cancel")
                                            .disabled(self.loading)
                                            .danger()
                                            .cursor_pointer()
                                            .label("Cancel")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.reset_otp_verification(Some(window), cx);
                                            })),
                                    )
                                })
                                .when(!self.otp_verification, |d| {
                                    d.child(
                                        Button::new("login")
                                            .disabled(self.loading)
                                            .primary()
                                            .cursor_pointer()
                                            .label("Login")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.sign_in(window, cx);
                                            })),
                                    )
                                })
                                .child(
                                    div()
                                        .id("go-to-sign-up")
                                        .text_center()
                                        .text_sm()
                                        .rounded_md()
                                        .when_else(
                                            self.loading || self.otp_verification,
                                            |d| d.text_color(cx.theme().muted_foreground),
                                            |d| {
                                                d.cursor_pointer().hover(|s| s.bg(cx.theme().muted))
                                            },
                                        )
                                        .child("Don't have an account? Sign up")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if !this.loading && !this.otp_verification {
                                                cx.open_url(api::SIGN_UP_URL);
                                            }
                                        })),
                                ),
                        ),
                ),
        )
    }
}
