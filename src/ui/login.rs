use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{InputState, OtpInput, OtpState, TextInput},
    ActiveTheme, Disableable, Icon, StyledExt,
};

use crate::auth::{Auth, AuthEvent, SessionState};

pub struct LoginUi {
    auth: Entity<Auth>,
    email_input: Entity<InputState>,
    otp_input: Entity<OtpState>,
    err_text: Option<String>,
    loading: bool,
    otp_verification: bool,
    _subscriptions: Vec<Subscription>,
}

impl LoginUi {
    fn new(auth: Entity<Auth>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let email_input = cx.new(|cx| InputState::new(window, cx).placeholder("user@email.com"));
        let otp_input = cx.new(|cx| OtpState::new(6, window, cx));

        let otp_sub = cx.subscribe(&otp_input, |this, _, event, cx| match event {
            gpui_component::input::InputEvent::Change => {
                this.verify_otp(cx);
            }
            _ => (),
        });

        Self {
            auth,
            email_input,
            otp_input,
            err_text: None,
            loading: false,
            otp_verification: false,
            _subscriptions: vec![otp_sub],
        }
    }

    pub fn view(auth: Entity<Auth>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, window, cx))
    }

    fn sign_in(&self, cx: &mut Context<Self>) {
        let email = self.email_input.read(cx).value();
        cx.spawn(async move |this, cx| {
            let _ = this.update(cx, |this, _| {
                this.loading = true;
            });

            let inner = this
                .read_with(cx, |this, cx| this.auth.read(cx).inner())
                .unwrap();

            let _inner = inner.clone();
            let result = cx
                .background_executor()
                .spawn(async move { inner.sign_in(&email).await })
                .await;

            let result = match result {
                Ok(email_idn) => {
                    cx.background_executor()
                        .spawn(async move { _inner.prepare_first_factor(&email_idn).await })
                        .await
                }
                Err(err) => {
                    this.update(cx, |this, _| {
                        this.loading = false;
                        this.err_text = Some(err.message);
                    })
                    .unwrap();
                    return;
                }
            };

            let _ = this.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(_) => {
                        this.otp_verification = true;
                        this.err_text = None;

                        this.otp_input.focus_handle(cx);
                    }
                    Err(err) => {
                        this.err_text = Some(format!("Failed to login: {err:?}"));
                    }
                };
            });
        })
        .detach();
    }

    fn verify_otp(&mut self, cx: &mut Context<Self>) {
        let code = self.otp_input.read(cx).value();
        if code.len() != 6 {
            return;
        }

        let code = code.clone();
        self.loading = true;

        cx.spawn(async move |this, cx| {
            let inner = this
                .read_with(cx, |this, cx| this.auth.read(cx).inner())
                .unwrap();

            let result = cx
                .background_executor()
                .spawn(async move { inner.attempt_first_factor(&code).await })
                .await;

            this.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(_) => {
                        this.reset_otp_verification(None, cx);

                        this.auth.update(cx, |_, cx| {
                            cx.emit(AuthEvent::Session(SessionState::SignedIn));
                        });
                    }
                    Err(err) => {
                        this.err_text = Some(err.message);
                    }
                };
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
        });
        self.otp_verification = false;
        self.email_input.update(cx, |state, cx| {
            state.focus_handle(cx);
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
                        .child(
                            div()
                                .flex()
                                .justify_center()
                                .items_center()
                                .bg(cx.theme().primary)
                                .rounded_md()
                                .size_6()
                                .child(
                                    Icon::new(Icon::empty())
                                        .text_color(cx.theme().primary_foreground)
                                        .size_5()
                                        .path("icons/cloud-moon.svg"),
                                ),
                        )
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
                                            TextInput::new(&self.email_input)
                                                .text_sm()
                                                .cleanable()
                                                .disabled(self.otp_verification)
                                                .line_clamp(1),
                                        )
                                        .when_some(self.err_text.clone(), |d, err_text| {
                                            d.child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().danger)
                                                    .child(err_text),
                                            )
                                        }),
                                )
                                .when(self.otp_verification, |d| {
                                    d.child(OtpInput::new(&self.otp_input)).child(
                                        div()
                                            .grid()
                                            .grid_cols(2)
                                            .gap_4()
                                            .child(
                                                Button::new("login-cancel")
                                                    .disabled(self.loading)
                                                    .danger()
                                                    .cursor_pointer()
                                                    .label("Cancel")
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.reset_otp_verification(
                                                                Some(window),
                                                                cx,
                                                            );
                                                        },
                                                    )),
                                            )
                                            .child(
                                                Button::new("login-verify")
                                                    .disabled(self.loading)
                                                    .primary()
                                                    .cursor_pointer()
                                                    .label("Verify")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.verify_otp(cx);
                                                    })),
                                            ),
                                    )
                                })
                                .when(!self.otp_verification, |d| {
                                    d.child(
                                        Button::new("login")
                                            .disabled(self.loading)
                                            .primary()
                                            .cursor_pointer()
                                            .label("Login")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.sign_in(cx);
                                            })),
                                    )
                                })
                                .child(
                                    div()
                                        .id("go-to-sign-up")
                                        .text_center()
                                        .text_sm()
                                        .child("Don't have an account? Sign up"),
                                ),
                        ),
                ),
        )
    }
}
