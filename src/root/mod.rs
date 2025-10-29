use gpui::{prelude::FluentBuilder, *};
use gpui_component::{notification::Notification, ActiveTheme, ContextModal, Icon, IconName, Root};
use header::Header;

use crate::{
    auth::{Auth, AuthClientEvent, AuthEvent, SessionState},
    err::AppError,
};

mod header;

actions!(window, [CloseWindow]);

pub struct Rooter {
    header: Entity<Header>,
    auth: Entity<Auth>,
    auth_loading: bool,
    session_state: Option<SessionState>,
    _subscriptions: Vec<Subscription>,
}

impl Rooter {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let header = Header::view(window, cx);
        let auth = cx.new(|cx| Auth::init(cx));

        let win_auth = auth.clone();
        cx.on_window_closed(move |cx| {
            win_auth.update(cx, |this, cx| {
                this.save(cx);
            });

            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            dbg!(event);
            match event {
                AuthEvent::Client(client_event) => match client_event {
                    AuthClientEvent::Loading => {
                        this.auth_loading = true;
                    }
                    AuthClientEvent::Setup(result) => match result {
                        Ok(_) => {
                            this.auth_loading = false;
                        }
                        Err(err) => {
                            this.auth_loading = false;
                            window.push_notification(
                                Notification::error(&err.message)
                                    .title("Failed to setup auth client")
                                    .autohide(true),
                                cx,
                            );
                        }
                    },
                },
                AuthEvent::Idle => {
                    this.auth_loading = false;
                }
                AuthEvent::Session(state) => {
                    match state {
                        SessionState::Validating => (),
                        _ => {
                            this.auth_loading = false;
                        }
                    };
                    this.session_state = Some(state.clone());
                }
            };
            cx.notify();
        });

        Self::setup_auth(cx);

        Self {
            header,
            auth,
            auth_loading: false,
            session_state: None,
            _subscriptions: vec![auth_sub],
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn setup_auth(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            // get inner client and prepare event
            let inner = this.update(cx, |this, cx| {
                this.auth.update(cx, |auth, cx| {
                    cx.emit(AuthEvent::Client(AuthClientEvent::Loading));
                    auth.inner()
                })
            });

            let inner = match inner {
                Ok(inner) => inner,
                Err(err) => {
                    this.update(cx, |this, cx| {
                        this.auth.update(cx, |_, cx| {
                            cx.emit(AuthEvent::Client(AuthClientEvent::Setup(Err(
                                AppError::gp_err(err),
                            ))));
                        });
                    })
                    .unwrap();
                    return;
                }
            };

            // setup
            let _inner = inner.clone();
            let result = cx
                .background_executor()
                .spawn(async move { _inner.setup_client().await })
                .await;

            // send event
            let is_ok = result.is_ok();
            this.update(cx, |this, cx| {
                this.auth.update(cx, |_, cx| {
                    cx.emit(AuthEvent::Client(AuthClientEvent::Setup(result)));

                    if is_ok {
                        cx.emit(AuthEvent::Session(SessionState::Validating));
                    }
                });
            })
            .unwrap();

            if is_ok {
                let _inner = inner.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move { _inner.fetch_token().await })
                    .await;

                this.update(cx, |this, cx| {
                    this.auth.update(cx, |_, cx| {
                        match result {
                            Ok(_) => cx.emit(AuthEvent::Session(SessionState::SignedIn)),
                            Err(_) => cx.emit(AuthEvent::Session(SessionState::LoggedOut)),
                        };
                    });
                })
                .unwrap();
            }
        })
        .detach();
    }
}

impl Render for Rooter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.header.clone())
            .when(self.auth_loading, |d| {
                d.child(
                    div().size_full().flex().child(
                        div()
                            .flex()
                            .w_full()
                            .justify_center()
                            .items_center()
                            .bg(cx.theme().background)
                            .p_6()
                            .gap_x_4()
                            .child(Icon::new(IconName::LoaderCircle).size_12().with_animation(
                                ElementId::CodeLocation(*std::panic::Location::caller()),
                                Animation::new(std::time::Duration::from_secs(2)).repeat(),
                                |el, delta| el.transform(Transformation::rotate(percentage(delta))),
                            ))
                            .when_none(&self.session_state, |d| d.child("Loading auth"))
                            .when_some(self.session_state.clone(), |d, _| {
                                d.child("Validating session")
                            }),
                    ),
                )
            })
            .when(!self.auth_loading, |d| {
                d.child(
                    div().flex().flex_row().flex_1().child(
                        div().flex_1().bg(cx.theme().background).p_6().child(
                            div()
                                .text_color(cx.theme().accent_foreground)
                                .text_lg()
                                .child("Main content area")
                                .when_some(self.session_state.clone(), |d, state| {
                                    d.child(match state {
                                        SessionState::SignedIn => "Signed in",
                                        SessionState::LoggedOut => "Logged out",
                                        _ => "...",
                                    })
                                }),
                        ),
                    ),
                )
            })
            .when(notification_layer.is_some(), |d| {
                d.child(notification_layer.unwrap())
            })
    }
}
