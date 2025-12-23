use std::sync::Arc;

use futures::StreamExt;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::collapsible::Collapsible;
use gpui_component::tooltip::Tooltip;
use gpui_component::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, StyledExt, ThemeMode, TitleBar,
    WindowExt,
    avatar::Avatar,
    button::{Button, ButtonVariants},
    h_flex,
    menu::DropdownMenu,
    notification::Notification,
    v_flex,
};
use uuid::Uuid;

use crate::entities::upload::UploadJobStatus;
use crate::{
    auth::{AuthEvent, AuthState, InnerAuth},
    entities::{
        UserData,
        nav::{NavEvent, NavState, Navigation},
        upload::{UploadJob, UploadManager, UploadUrlState},
    },
    rt,
    theme::*,
    ui::{
        _components::{app_icon, create_space_dialog, loading_icon, select_space_dialog},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api::{self, models::cloud::res::InitiateUploadResponse},
};

actions!(header, [EmptyAction, ClearJobs]);

pub struct JobViewCollapsible {
    opened: Vec<bool>,
}

pub struct HeaderUi {
    auth: AuthState,
    user_data: Entity<UserData>,
    nav: Navigation,
    upload_manager: Entity<UploadManager>,

    logged_in: bool,
    loading_spaces: bool,
    loading_user: bool,
    creating_space: bool,
    upload_running: bool,
    job_view_collapsible: Entity<JobViewCollapsible>,

    _subscriptions: Vec<Subscription>,
}

impl HeaderUi {
    pub fn new(
        auth: AuthState,
        user_data: Entity<UserData>,
        upload_manager: Entity<UploadManager>,
        nav: Navigation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let job_view_collapsible = cx.new(|_cx| JobViewCollapsible { opened: Vec::new() });

        let upload_sub = cx.subscribe_in(&upload_manager, window, |this, _, _ev, window, cx| {
            let opened = this
                .upload_manager
                .read(cx)
                .jobs()
                .iter()
                .map(|_| false)
                .collect::<Vec<_>>();

            this.job_view_collapsible.update(cx, |jb, cx| {
                jb.opened = opened;
                cx.notify();
            });

            this.process_upload(window, cx);
        });

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                crate::auth::AuthEvent::Session(session_state) => match session_state {
                    crate::auth::SessionState::SignedIn => {
                        this.logged_in = true;

                        this.fetch_user(window, cx);
                        this.fetch_spaces(None, window, cx);
                    }
                    crate::auth::SessionState::LoggedOut => {
                        this.user_data.update(cx, |state, cx| {
                            state.reset();
                            cx.notify();
                        });
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
                _ => (),
            };
        });

        Self {
            auth,
            user_data,
            nav,
            upload_manager,
            logged_in: false,
            loading_spaces: false,
            loading_user: false,
            creating_space: false,
            upload_running: false,
            job_view_collapsible,
            _subscriptions: vec![auth_sub, nav_sub, upload_sub],
        }
    }

    pub fn view(
        auth: AuthState,
        user_data: Entity<UserData>,
        upload_manager: Entity<UploadManager>,
        nav: Navigation,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, user_data, upload_manager, nav, window, cx))
    }

    fn change_mode(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();

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
                    Ok(user) => this.user_data.update(cx, |ctx, _cx| ctx.user = Some(user)),
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
                                        this.user_data.clone(),
                                        this.nav.clone(),
                                        this.upload_manager.clone(),
                                        NavState::new(user_space.space.id, user_space.folder),
                                        window,
                                        cx,
                                    ),
                                    cx,
                                );
                            });
                        }

                        this.user_data.update(cx, |ctx, _cx| {
                            ctx.user_spaces = user_spaces.into_iter().map(Arc::new).collect()
                        });
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

    fn logout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();

        let task = rt::spawn(cx, async move { inner.sign_out().await });

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(_) => this.auth.update(cx, |auth, cx| {
                        auth.save(cx);
                        cx.emit(AuthEvent::Session(crate::auth::SessionState::LoggedOut));
                    }),
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to log out"),
                        cx,
                    ),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn process_upload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.upload_running {
            return;
        }

        self.upload_running = true;
        cx.notify();

        tracing::info!(msg = "starting upload loop");

        cx.spawn_in(window, async move |this, cx| {
            loop {
                let jobs_result = this.read_with(cx, |this, cx| {
                    this.upload_manager.read(cx).next(cx).cloned()
                });

                let Ok(inner) = this.read_with(cx, |this, cx| this.auth.read(cx).inner()) else {
                    break;
                };

                match jobs_result {
                    Ok(jobs) => match jobs {
                        Some(job) => {
                            let _ = this.update_in(cx, |_this, window, cx| {
                                window.push_notification(Notification::info("Upload started"), cx);
                                job.clone().update(cx, |job, cx| {
                                    job.status = UploadJobStatus::InProgress;
                                    cx.notify();
                                });
                            });

                            Self::initialize_uploads(inner.clone(), job.clone(), cx).await;

                            Self::upload_files(inner, job.clone(), cx).await;

                            let _ = this.update(cx, |this, cx| {
                                let space_id = job.read(cx).space_id.clone();
                                let folder_id = job.read(cx).folder.id.clone();

                                this.nav.update(cx, |_nav, cx| {
                                    cx.emit(NavEvent::RefreshView(NavState::new(
                                        space_id.clone(),
                                        folder_id,
                                    )));
                                });

                                job.clone().update(cx, |job, cx| {
                                    if job
                                        .uploads
                                        .iter()
                                        .all(|u| matches!(u.url_state, UploadUrlState::Done))
                                    {
                                        job.status = UploadJobStatus::Done;
                                    }
                                    cx.notify();
                                });
                            });
                        }
                        None => break,
                    },
                    Err(err) => {
                        let _ = this.update_in(cx, |_this, window, cx| {
                            window.push_notification(
                                Notification::error(format!("{err}"))
                                    .title("Failed to handle upload jobs"),
                                cx,
                            );
                        });

                        break;
                    }
                };
            }

            tracing::info!(msg = "No jobs.. loop complete");
            let _ = this.update(cx, |this, cx| {
                this.upload_running = false;
                cx.notify();
            });
        })
        .detach();
    }

    async fn initialize_uploads(
        inner: Arc<InnerAuth>,
        job: Entity<UploadJob>,
        cx: &mut AsyncWindowContext,
    ) {
        let tasks = job
            .read_with(cx, |this, cx| {
                let job = job.clone();
                let folder_id = this.folder.id;
                let space_id = this.space_id;

                this.uploads
                    .iter()
                    .enumerate()
                    .map(|(i, state)| {
                        let inner = inner.clone();
                        let job = job.clone();

                        let file_name = state
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_owned())
                            .unwrap_or_default();

                        let task = rt::spawn(cx, async move {
                            inner
                                .get_token()
                                .await
                                .map_async(async move |token| {
                                    api::cloud::init_file_upload(
                                        &token, &space_id, &folder_id, &file_name,
                                    )
                                    .await
                                })
                                .await
                        });

                        cx.spawn(async move |cx| {
                            let result = task.await.flatten();
                            let _ = job.update(cx, |job, cx| {
                                job.uploads.get_mut(i).map(|u| match result {
                                    Ok(data) => u.url_state = UploadUrlState::Uploading(data),
                                    Err(err) => u.url_state = UploadUrlState::Error(err),
                                });
                                cx.notify();
                            });
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        cx.spawn(async move |_cx| {
            let _ = futures::stream::iter(tasks)
                .buffer_unordered(8)
                .collect::<Vec<_>>()
                .await;
        })
        .await;
    }

    async fn upload_files(
        inner: Arc<InnerAuth>,
        job: Entity<UploadJob>,
        cx: &mut AsyncWindowContext,
    ) {
        let tasks = job
            .update(cx, |this, cx| {
                let folder_id = this.folder.id;
                let space_id = this.space_id;

                this.uploads
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(i, state)| {
                        let job = job.clone();
                        let inner = inner.clone();
                        let path = state.path.clone();

                        if let UploadUrlState::Uploading(data) = &state.url_state {
                            let InitiateUploadResponse { url, file_name } = data.clone();

                            let task = rt::spawn(cx, async move {
                                api::upload(url.as_str(), path)
                                    .await
                                    .map_async(async move |data| {
                                        inner.get_token().await.map(|token| (token, data.0, data.1))
                                    })
                                    .await
                                    .map_async(async move |(token, file_size, updated_millis)| {
                                        api::cloud::complete_file_upload(
                                            &token,
                                            &space_id,
                                            &folder_id,
                                            file_name.as_str(),
                                            file_size,
                                            updated_millis,
                                        )
                                        .await
                                    })
                                    .await
                            });

                            Some(cx.spawn(async move |_this, cx| {
                                let result = task.await.flatten();
                                let _ = job.update(cx, |job, cx| {
                                    job.uploads.get_mut(i).map(|u| match result {
                                        Ok(_) => u.url_state = UploadUrlState::Done,
                                        Err(err) => u.url_state = UploadUrlState::Error(err),
                                    });
                                    cx.notify();
                                });
                            }))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        cx.spawn(async move |_cx| {
            let _ = futures::stream::iter(tasks)
                .buffer_unordered(8)
                .collect::<Vec<_>>()
                .await;
        })
        .await;
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
            .when(self.logged_in, |this| {
                this.child(self.render_space_switcher(cx))
            })
            .child(
                div()
                    .on_action(cx.listener(|this, _: &ClearJobs, _window, cx| {
                        this.upload_manager.update(cx, |um, cx| {
                            um.trim_completed(cx);
                            cx.notify();
                        });
                    }))
                    .pr(px(5.0))
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(self.render_upload_popup(cx))
                    .when(self.logged_in, |this| {
                        this.child(self.render_user_popup(cx))
                    })
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
            .map(|this| match self.user_data.read(cx).user.as_ref() {
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
                    .dropdown_menu(move |menu, _window, _cx| {
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
                                            this.logout(window, cx);
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
        let current_space = self.nav.read(cx).current_space_id().and_then(|sp_id| {
            self.user_data
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
        let entity = cx.weak_entity();

        h_flex()
            .gap_2()
            .child(
                Button::new("space_switcher")
                    .small()
                    .compact()
                    .icon(Icon::new(IconName::GalleryVerticalEnd))
                    .dropdown_caret(true)
                    .map(|this| match current_space.clone() {
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
                                .read_with(cx, |this, cx| {
                                    this.user_data.read(cx).user_spaces.clone()
                                })
                                .unwrap_or_default();

                            select_space_dialog::comp(
                                dialog,
                                entity,
                                user_spaces,
                                cx,
                                |entity, state, window, cx| {
                                    entity
                                        .clone()
                                        .update(cx, |this, cx| {
                                            this.nav.update(cx, |stack, cx| {
                                                if let Some(current_space_id) =
                                                    stack.current_space_id()
                                                    && current_space_id == state.space_id()
                                                {
                                                    // skip
                                                    return;
                                                }

                                                stack.push(
                                                    BrowseUi::view(
                                                        this.auth.clone(),
                                                        this.user_data.clone(),
                                                        this.nav.clone(),
                                                        this.upload_manager.clone(),
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
                    }),
            )
            .when_some(current_space, |this, us| {
                this.child(
                    Button::new("members")
                        .icon(Icon::empty().path("icons/users.svg"))
                        .small()
                        .disabled(self.loading_spaces)
                        .on_click(cx.listener(move |_this, _ev, window, cx| {
                            cx.stop_propagation();
                            let us = us.clone();

                            window.open_sheet(cx, move |sheet, _window, _cx| {
                                sheet.child(
                                    v_flex()
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_medium()
                                                .child(us.space.name.clone()),
                                        )
                                        .child(div().child("Members")),
                                )
                            });
                        })),
                )
            })
    }

    fn render_upload_popup(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.weak_entity();

        Button::new("upld")
            .icon(Icon::empty().path("icons/upload.svg"))
            .small()
            .outline()
            .mr_1()
            .on_click(|_ev, _window, cx| {
                cx.stop_propagation();
            })
            .dropdown_menu(move |menu, _window, cx| {
                let entity = entity.clone();

                let (jobs, collapsible) = entity
                    .read_with(cx, |this, cx| {
                        (
                            this.upload_manager.read(cx).jobs().clone(),
                            this.job_view_collapsible.clone(),
                        )
                    })
                    .unwrap();

                let mut menu = menu.max_w(px(512.));
                if jobs.is_empty() {
                    return menu.menu("No upload jobs", Box::new(EmptyAction));
                }

                for (i, job) in jobs.into_iter().enumerate() {
                    let collapsible = collapsible.clone();
                    menu = menu
                        .menu_element(Box::new(EmptyAction), move |_window, cx| {
                            let cl = collapsible.clone();

                            Collapsible::new()
                                .min_w_112()
                                .gap_2()
                                .open(
                                    collapsible
                                        .read(cx)
                                        .opened
                                        .get(i)
                                        .cloned()
                                        .unwrap_or_default(),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap_2()
                                        .child(format!(
                                            "{}: {}",
                                            match job.read(cx).status {
                                                UploadJobStatus::Queued => "Queued",
                                                UploadJobStatus::InProgress => "Uploading",
                                                UploadJobStatus::Done => "Completed",
                                            },
                                            job.read(cx).folder.path.clone()
                                        ))
                                        .child(
                                            Button::new(SharedString::new(format!("btn-cl-{i}")))
                                                .icon(IconName::ChevronsUpDown)
                                                .small()
                                                .outline()
                                                .on_click(move |_ev, _window, cx| {
                                                    cx.stop_propagation();

                                                    cl.update(cx, |cl, cx| {
                                                        cl.opened.get_mut(i).map(|o| *o = !*o);
                                                        cx.notify();
                                                    });
                                                }),
                                        ),
                                )
                                .content(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .max_h_112()
                                        .min_w_112()
                                        .gap_2()
                                        .child(Self::render_job_menu_view(i, job.clone(), cx))
                                        .child(div().child(format!(
                                            "Total items: {}",
                                            job.read(cx).uploads.len()
                                        ))),
                                )
                        })
                        .separator();
                }
                menu.separator()
                    .menu_element(Box::new(ClearJobs), move |_window, _cx| {
                        div()
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(Icon::new(IconName::Close))
                            .child("Clear all")
                    })
            })
    }

    fn render_job_menu_view(i: usize, job: Entity<UploadJob>, cx: &App) -> impl IntoElement {
        div()
            .id(SharedString::new(i.to_string()))
            .h_full()
            .overflow_y_scroll()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .justify_start()
            .children(job.read(cx).uploads.iter().map(|state| {
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .py_0p5()
                    .rounded_l_lg()
                    .bg(cx.theme().sidebar)
                    .child(
                        div()
                            .text_sm()
                            .max_w_64()
                            .truncate()
                            .text_ellipsis()
                            .whitespace_normal()
                            .child(
                                state
                                    .path
                                    .file_name()
                                    .and_then(|f| f.to_str())
                                    .map(|s| s.to_owned())
                                    .unwrap_or_default(),
                            ),
                    )
                    .map(|this| match &state.url_state {
                        UploadUrlState::Queued => this.child(loading_icon(|icon| icon.size_4())),
                        UploadUrlState::Uploading(_) => {
                            this.child(Icon::empty().path("icons/upload.svg"))
                        }
                        UploadUrlState::Done => {
                            this.child(Icon::empty().path("icons/check.svg").text_color(green()))
                        }
                        UploadUrlState::Error(err) => {
                            let message = err.message.clone();
                            this.child(
                                div()
                                    .id(SharedString::from(format!("err-{i}")))
                                    .child(
                                        Icon::new(IconName::CircleX).text_color(cx.theme().danger),
                                    )
                                    .tooltip(move |window, cx| {
                                        Tooltip::new(message.clone()).build(window, cx)
                                    }),
                            )
                        }
                    })
            }))
    }

    fn render_nav_buttons(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_0p5()
            .when(self.logged_in, |this| {
                this.child(
                    Button::new("go-back")
                        .icon(Icon::new(IconName::ArrowLeft))
                        .small()
                        .ghost()
                        .disabled(self.nav.read(cx).at_begining())
                        .on_click(cx.listener(|this, _ev, window, cx| {
                            cx.stop_propagation();
                            window.close_sheet(cx);

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
                        .on_click(cx.listener(|this, _ev, window, cx| {
                            cx.stop_propagation();
                            window.close_sheet(cx);

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
                        .on_click(cx.listener(|this, _ev, window, cx| {
                            cx.stop_propagation();
                            window.close_sheet(cx);

                            this.nav.update(cx, |_nav, cx| {
                                cx.emit(NavEvent::Refresh);
                            });
                        })),
                )
            })
            .child(
                app_icon::comp(cx, |icon| icon.size_4())
                    .p_1()
                    .id("home")
                    .ml_1()
                    .hover(|el| el.bg(cx.theme().primary_hover))
                    .on_click(cx.listener(|this, _ev, window, cx| {
                        cx.stop_propagation();
                        window.close_sheet(cx);

                        this.nav.update(cx, |stack, cx| {
                            stack.home(cx);
                        })
                    })),
            )
    }
}
