use gpui::{prelude::FluentBuilder, *};
use gpui_component::{ActiveTheme, Root, WindowExt, notification::Notification};
use header::HeaderUi;

use crate::{
    auth::{Auth, AuthClientEvent, AuthEvent, AuthState, SessionState},
    entities::{UserData, nav::NavStack, transfer::TransferManager},
    rt,
    ui::{home::HomeUi, login::LoginUi},
};

mod _components;
mod header;
mod home;
mod login;

actions!(window, [CloseWindow, Quit]);
pub const APP_CONTEXT: &str = "Rooter";

fn init_kb(cx: &mut App) {
    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new("cmd-w", CloseWindow, Some(APP_CONTEXT))]);

    #[cfg(target_os = "macos")]
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, Some(APP_CONTEXT))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("ctrl-w", CloseWindow, Some(APP_CONTEXT))]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([KeyBinding::new("alt-f4", Quit, Some(APP_CONTEXT))]);
}

pub struct Rooter {
    focus_handle: FocusHandle,

    header_ui: Entity<HeaderUi>,
    login_ui: Entity<LoginUi>,
    home_ui: Entity<HomeUi>,

    auth: AuthState,
    auth_loading: bool,
    session_state: Option<SessionState>,
    _subscriptions: Vec<Subscription>,
}

impl Rooter {
    pub fn new(focus_handle: FocusHandle, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let auth = cx.new(|cx| Auth::init(cx));
        let user_state = cx.new(|_| UserData::new());
        let nav = cx.new(|_| NavStack::new());
        let transfer_manager = cx.new(|_cx| TransferManager::new());

        let header_ui = HeaderUi::view(
            auth.clone(),
            user_state.clone(),
            transfer_manager.clone(),
            nav.clone(),
            window,
            cx,
        );
        let login_ui = LoginUi::view(auth.clone(), window, cx);
        let home_ui = HomeUi::view(
            auth.clone(),
            user_state.clone(),
            transfer_manager.clone(),
            nav.clone(),
            window,
            cx,
        );

        let _auth = auth.clone();
        cx.on_window_closed(move |cx| {
            _auth.update(cx, |auth, cx| {
                auth.save(cx);
                cx.notify();
            });
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            tracing::info!(msg = "Auth event", event = format!("{event}"));
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

        Self {
            focus_handle,
            header_ui,
            login_ui,
            home_ui,
            auth,
            auth_loading: false,
            session_state: None,
            _subscriptions: vec![auth_sub],
        }
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            init_kb(cx);

            let focus_handle = cx.focus_handle();
            focus_handle.focus(window);

            let root = Self::new(focus_handle, window, cx);
            root.setup_auth(window, cx);
            root
        })
    }

    fn setup_auth(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            // get inner client and prepare event
            let inner = this
                .update(cx, |this, cx| {
                    this.auth.update(cx, |auth, cx| {
                        cx.emit(AuthEvent::Client(AuthClientEvent::Loading));
                        auth.inner()
                    })
                })
                .unwrap();

            let _inner = inner.clone();
            let client_result = rt::spawn(cx, async move { _inner.setup_client().await })
                .unwrap()
                .await
                .flatten();

            // send event
            let is_ok = client_result.is_ok();
            this.update(cx, |this, cx| {
                this.auth.update(cx, |_, cx| {
                    cx.emit(AuthEvent::Client(AuthClientEvent::Setup(client_result)));

                    if is_ok {
                        cx.emit(AuthEvent::Session(SessionState::Validating));
                    }
                });
            })
            .unwrap();

            if is_ok {
                let _inner = inner.clone();
                let result = rt::spawn(cx, async move { _inner.fetch_token().await })
                    .unwrap()
                    .await
                    .flatten();

                let has_session = rt::spawn(cx, async move { inner.has_session().await })
                    .unwrap()
                    .await
                    .unwrap();

                this.update_in(cx, |this, window, cx| {
                    this.auth.update(cx, |_, cx| {
                        match result {
                            Ok(_) => cx.emit(AuthEvent::Session(SessionState::SignedIn)),
                            Err(err) => {
                                window.push_notification(
                                    Notification::error(err.message).autohide(false),
                                    cx,
                                );

                                if has_session {
                                    cx.emit(AuthEvent::Session(SessionState::SignedIn));
                                } else {
                                    cx.emit(AuthEvent::Session(SessionState::LoggedOut));
                                }
                            }
                        };
                    });
                })
                .unwrap();
            }
        })
        .detach();

        let inner = self.auth.read(cx).inner();
        cx.spawn_in(window, async move |this, cx| {
            loop {
                Timer::after(std::time::Duration::from_secs(40)).await;

                let is_logged_out = this
                    .read_with(cx, |this, _cx| {
                        this.session_state
                            .as_ref()
                            .map(|s| matches!(s, SessionState::LoggedOut))
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();

                if is_logged_out {
                    continue;
                }

                let _inner = inner.clone();

                let result = rt::spawn(cx, async move { _inner.fetch_token().await })
                    .unwrap()
                    .await
                    .flatten();

                if let Err(err) = result {
                    let _ = this.update_in(cx, |_this, window, cx| {
                        window.push_notification(
                            Notification::error(err.message).title("Failed to refresh auth"),
                            cx,
                        );
                    });

                    let _inner = inner.clone();
                    let result = rt::spawn(cx, async move { _inner.has_session().await })
                        .unwrap()
                        .await;
                    if let Ok(has_session) = result
                        && !has_session
                    {
                        let _ = this.update(cx, |this, cx| {
                            this.auth.update(cx, |auth, cx| {
                                auth.save(cx);
                                cx.emit(AuthEvent::Session(SessionState::LoggedOut));
                            });
                            cx.notify();
                        });
                    }
                }
            }
        })
        .detach();
    }

    fn on_close_or_quit(&mut self, cx: &mut Context<Self>) {
        self.auth.update(cx, |auth, cx| {
            auth.save(cx);
        });
    }
}

impl Render for Rooter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notification_layer = Root::render_notification_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);

        div()
            .id("rooter")
            .key_context(APP_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &CloseWindow, window, cx| {
                this.on_close_or_quit(cx);
                window.remove_window();
            }))
            .on_action(cx.listener(|this, _: &Quit, _window, cx| {
                this.on_close_or_quit(cx);
                cx.quit();
            }))
            .size_full()
            .child(self.header_ui.clone())
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
                            .child(_components::loading_icon(|icon| icon.size_8()))
                            .when_none(&self.session_state, |d| d.child("Loading auth"))
                            .when_some(self.session_state.clone(), |d, _| {
                                d.child("Validating session")
                            }),
                    ),
                )
            })
            .when(!self.auth_loading, |d| {
                d.when_some(self.session_state.clone(), |d, state| match state {
                    SessionState::SignedIn => d.child(self.home_ui.clone()),
                    SessionState::LoggedOut => d.child(self.login_ui.clone()),
                    SessionState::Validating => d.child(
                        div().size_full().flex().child(
                            div()
                                .flex()
                                .w_full()
                                .justify_center()
                                .items_center()
                                .bg(cx.theme().background)
                                .p_6()
                                .gap_x_4()
                                .child(_components::loading_icon(|icon| icon.size_8()))
                                .child("Syncing session"),
                        ),
                    ),
                })
            })
            .when_some(notification_layer, |d, layer| d.child(layer))
            .when_some(dialog_layer, |d, layer| d.child(layer))
            .when_some(sheet_layer, |d, layer| d.child(layer))
    }
}
