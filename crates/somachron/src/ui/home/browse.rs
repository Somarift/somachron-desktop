use std::{collections::HashMap, ops::Range, path::PathBuf, str::FromStr, sync::Arc};

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Side, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    h_flex,
    label::Label,
    menu::DropdownMenu,
    notification::Notification,
    scroll::ScrollableElement,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};
use url::Url;
use uuid::Uuid;

use crate::{
    auth::AuthState,
    entities::{
        UserData,
        bounds::RenderBounds,
        media::{
            ElementType, FetchMedia, MEDIA_GAP, MEDIA_HEIGHT, MediaAssetState, MediaState,
            PreviewAssetType, SECTION_HEIGHT,
        },
        nav::{NavEvent, NavId, NavState, Navigation},
        upload::{UploadJob, UploadManager},
    },
    err::AppError,
    rt,
    store::paths,
    ui::{
        _components::{self, create_folder_dialog, delete_dialog, loading_icon},
        header::EmptyAction,
        home::media::MediaUi,
    },
    util::MapAsync,
    web::api::{
        self,
        models::cloud::{
            MediaType,
            res::{FileMetaReponse, FolderResponse},
        },
    },
};

pub struct BrowseUi {
    auth: AuthState,
    user_data: Entity<UserData>,
    nav: Navigation,
    upload_manager: Entity<UploadManager>,
    render_bounds: Entity<RenderBounds>,

    current_nav: NavState,
    folder: Option<FolderResponse>,
    folders: Vec<FolderResponse>,
    media_state: Entity<MediaState>,
    file_checked: HashMap<Uuid, SharedString>,

    visible_item_range: Range<usize>,
    files_scroll_handle: ScrollHandle,
    header_scroll_handle: ScrollHandle,

    loading_folders: bool,
    loading_files: bool,
    creating_folder: bool,
    deleting: bool,
    _subscriptions: Vec<Subscription>,
}

impl BrowseUi {
    fn new(
        auth: AuthState,
        user_data: Entity<UserData>,
        nav: Navigation,
        upload_manager: Entity<UploadManager>,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let media_state = cx.new(|_cx| MediaState::new());
        let render_bounds = cx.new(|_cx| RenderBounds::new());

        let size_sub = cx.subscribe_in(
            &render_bounds,
            window,
            |this, _entity, _event, _window, cx| {
                Self::compute_visible_grid(this, cx);
                Self::update_visible_state(this, cx);
            },
        );

        let nav_sub = cx.subscribe_in(&nav, window, |this, _entity, event, window, cx| {
            let (refresh, fetch_files) = match event {
                NavEvent::Refresh => (true, true),
                NavEvent::RefreshView(nav_state) => (true, &this.current_nav == nav_state),
                _ => (false, false),
            };

            if refresh {
                if fetch_files {
                    this.reset_state(cx);
                    this.fetch_files(window, cx);
                }

                this.fetch_folders(window, cx);
            }
        });

        let media_sub =
            cx.subscribe_in(&media_state, window, |this, _entity, event, window, cx| {
                let FetchMedia { file } = event;
                this.fetch_image_urls(window, cx, file.clone());
            });

        Self {
            auth,
            user_data,
            nav,
            render_bounds,
            current_nav,
            upload_manager,
            folder: None,
            folders: Vec::new(),
            media_state,
            file_checked: HashMap::new(),
            visible_item_range: 0..0,
            files_scroll_handle: ScrollHandle::new(),
            header_scroll_handle: ScrollHandle::new(),
            loading_folders: false,
            loading_files: false,
            creating_folder: false,
            deleting: false,
            _subscriptions: vec![size_sub, nav_sub, media_sub],
        }
    }

    pub fn view(
        auth: AuthState,
        user_data: Entity<UserData>,
        nav: Navigation,
        upload_manager: Entity<UploadManager>,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let mut entity = Self::new(
                auth,
                user_data,
                nav,
                upload_manager,
                current_nav.for_browse(),
                window,
                cx,
            );
            entity.fetch_folders(window, cx);
            entity.fetch_files(window, cx);
            entity
        })
    }

    fn reset_state(&mut self, cx: &mut Context<Self>) {
        self.folders.clear();
        self.folder = None;
        self.media_state.update(cx, |ms, cx| {
            ms.clear();
            cx.notify();
        });
        self.file_checked.clear();
        self.visible_item_range = 0..0;
    }

    fn compute_visible_grid(this: &mut Self, cx: &mut Context<Self>) {
        this.render_bounds.update(cx, |bounds, cx| {
            this.media_state.update(cx, |md, cx| {
                md.compute_visible_grid(bounds);
                cx.notify();
            });
            cx.notify();
        });

        this.visible_item_range = this.render_bounds.read_with(cx, |bounds, _cx| {
            this.media_state.read(cx).get_initial_visible_range(bounds)
        });

        cx.notify();
    }

    fn update_visible_state(this: &mut Self, cx: &mut Context<Self>) {
        let offset = this.files_scroll_handle.offset().y;
        let scroll_offset = offset.abs();

        let bounds = this.render_bounds.read(cx);
        let end_offset = scroll_offset + bounds.height;

        this.visible_item_range = this
            .media_state
            .read(cx)
            .get_visible_state(scroll_offset, end_offset);

        cx.notify();
    }

    fn fetch_folders(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.current_nav.clone();
        let _state = state.clone();

        let inner = self.auth.read(cx).inner();

        let _inner = inner.clone();
        let folders = rt::spawn(cx, async move {
            _inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::list_folders(&token, _state.space_id(), _state.folder_id()).await
                })
                .await
        });

        let folder = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::get_folder(&token, state.space_id(), state.folder_id()).await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_folders = true;
                cx.notify();
            });

            let folders = folders.await.flatten();
            let folder = folder.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_folders = false;
                match folders {
                    Ok(folders) => {
                        this.folders = folders;
                    }
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to fetch folders"),
                        cx,
                    ),
                };

                match folder {
                    Ok(folder) => this.folder = Some(folder),
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to get current folder"),
                        cx,
                    ),
                };

                cx.notify();
            });
        })
        .detach();
    }

    fn fetch_files(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.current_nav.clone();

        let inner = self.auth.read(cx).inner();

        let files = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::list_files(&token, state.space_id(), state.folder_id()).await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_files = true;
                cx.notify();
            });

            let files = files.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_files = false;
                match files {
                    Ok(files) => {
                        this.media_state.update(cx, |ms, cx| {
                            ms.set_files(files);
                            cx.notify();
                        });

                        Self::compute_visible_grid(this, cx);
                    }
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to get files"),
                        cx,
                    ),
                };

                Self::update_visible_state(this, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn fetch_image_urls(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        file: Arc<FileMetaReponse>,
    ) {
        let nav_state = self.current_nav.clone();

        let file = self.media_state.update(cx, |ms, cx| {
            if let Some(state) = ms.asset_mut(&file.id) {
                match state {
                    MediaAssetState::Queued
                    | MediaAssetState::Error
                    | MediaAssetState::Loaded { .. } => {
                        return None;
                    }
                    _ => (),
                };

                *state = MediaAssetState::Queued;
                cx.notify();

                return Some(file);
            }
            None
        });

        let Some(file) = file else {
            return;
        };

        let inner = self.auth.read(cx).inner();

        let _file = file.clone();
        let task = rt::spawn(cx, async move {
            let cache_dir = paths::cache_dir().map_err(|err| AppError::err(err))?;
            let thumbnail_file = cache_dir.join(format!(
                "thumbnail_{}_{}",
                _file.id,
                _file.updated_at.timestamp_millis()
            ));

            if thumbnail_file.exists() {
                return Ok(thumbnail_file);
            }

            let file_id = _file.id.clone();
            let th_path = thumbnail_file.clone();
            let result = inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::get_thumbnail_stream_url(&token, nav_state.space_id(), &file_id)
                        .await
                })
                .await
                .map_async(async move |urls| api::download(urls.url, th_path).await)
                .await;

            if let Err(_) = result {
                let _ = tokio::fs::remove_file(&thumbnail_file).await;
            }

            result
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(th_path) => {
                        this.media_state.update(cx, |ms, _cx| {
                            ms.asset_mut(&file.id).map(|u| {
                                *u = MediaAssetState::Loaded {
                                    thumbnail_path: th_path.clone(),
                                    preview_asset_ty: PreviewAssetType::Loading,
                                };
                            });
                        });

                        this.__fetch_preview_url(window, cx, file, th_path);
                    }
                    Err(err) => {
                        this.media_state.update(cx, |ms, _cx| {
                            ms.asset_mut(&file.id).map(|u| {
                                *u = MediaAssetState::Error;
                            });
                        });
                        window.push_notification(Notification::error(err.message), cx);
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn __fetch_preview_url(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        file: Arc<FileMetaReponse>,
        thumbnail_path: PathBuf,
    ) {
        let nav_state = self.current_nav.clone();
        let inner = self.auth.read(cx).inner();

        let _file = file.clone();
        let task = rt::spawn(cx, async move {
            let cache_dir = paths::cache_dir().map_err(|err| AppError::err(err))?;

            let preview_file = cache_dir.join(format!(
                "preview_{}_{}",
                _file.id,
                _file.updated_at.timestamp_millis()
            ));

            if _file.media_type == MediaType::Image && preview_file.exists() {
                return Ok(PreviewAssetType::Preview(preview_file));
            }

            let file_id = _file.id.clone();
            let pr_path = preview_file.clone();
            let result = inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::get_preview_stream_url(&token, nav_state.space_id(), &file_id).await
                })
                .await
                .map_async(async move |urls| match _file.media_type {
                    MediaType::Image => api::download(urls.url, pr_path)
                        .await
                        .map(PreviewAssetType::Preview),
                    MediaType::Video => {
                        let url = Url::from_str(&urls.url).map_err(|err| AppError::err(err))?;
                        Ok(PreviewAssetType::VideoUrl(url))
                    }
                })
                .await;

            if let Err(_) = result {
                let _ = tokio::fs::remove_file(&preview_file).await;
            }

            result
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(pr_ty) => {
                        this.media_state.update(cx, |ms, _cx| {
                            ms.asset_mut(&file.id).map(|u| {
                                *u = MediaAssetState::Loaded {
                                    thumbnail_path,
                                    preview_asset_ty: pr_ty,
                                };
                            });
                        });
                    }
                    Err(err) => {
                        window.push_notification(Notification::error(err.message), cx);
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn open_media(this: &mut Self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        this.nav.update(cx, |nav, cx| {
            nav.push(
                MediaUi::view(
                    this.current_nav.clone(),
                    this.media_state.clone(),
                    index,
                    window,
                    cx,
                ),
                cx,
            );
            cx.notify();
        });
    }

    fn open_upload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(SharedString::new_static("Select files to upload")),
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = paths.await;

            let result = match result {
                Ok(r) => r,
                Err(err) => {
                    let _ = this.update_in(cx, |_this, window, cx| {
                        window.push_notification(
                            Notification::warning(format!("{err}")).title("No file selected"),
                            cx,
                        );
                    });
                    return;
                }
            };

            match result {
                Ok(Some(paths)) => {
                    let paths = paths
                        .into_iter()
                        .filter(|p| {
                            p.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| {
                                    [
                                        "jpg", "JPG", "jpeg", "JPEG", "HEIC", "heic", "MOV", "mov",
                                        "mp4", "MP4", "mpeg", "MPEG", "png", "PNG",
                                    ]
                                    .contains(&e)
                                })
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>();

                    let _ = this.update_in(cx, move |_this, window, cx| {
                        let paths = Arc::new(paths);

                        if paths.is_empty() {
                            window.push_notification(
                                Notification::info("No media files selected"),
                                cx,
                            );
                        } else {
                            let entity = cx.weak_entity();

                            window.open_dialog(cx, move |dialog, _window, cx| {
                                let entity = entity.clone();
                                let paths = paths.clone();

                                dialog
                                    .alert()
                                    .keyboard(false)
                                    .overlay_closable(false)
                                    .rounded_lg()
                                    .title("Upload")
                                    .v_flex()
                                    .max_h_128()
                                    .child(
                                        div()
                                            .id("upload-d")
                                            .overflow_y_scroll()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .children(paths.iter().cloned().map(|p| {
                                                div()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded_md()
                                                    .bg(cx.theme().sidebar)
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Icon::new(IconName::File).small())
                                                    .text_sm()
                                                    .child(Label::new(
                                                        p.file_name()
                                                            .and_then(|s| s.to_str())
                                                            .map(|s| s.to_owned())
                                                            .unwrap_or_default(),
                                                    ))
                                            })),
                                    )
                                    .footer(move |_, _, _, _| {
                                        let entity = entity.clone();
                                        let paths = paths.clone();

                                        let cancel = Button::new("upld-cancel")
                                            .label("Cancel")
                                            .on_click(|_, window, cx| {
                                                window.close_dialog(cx);
                                            });

                                        let submit = Button::new("upld-sbt")
                                            .primary()
                                            .label("Upload")
                                            .on_click(move |_ev, window, cx| {
                                                window.close_dialog(cx);

                                                let entity = entity.clone();
                                                let paths = paths.clone();
                                                let _ = entity.update(cx, |this, cx| {
                                                    if let Some(folder) = this.folder.as_ref() {
                                                        this.upload_manager.update(cx, |um, cx| {
                                                            um.push(
                                                                UploadJob::new(
                                                                    paths,
                                                                    this.current_nav
                                                                        .space_id()
                                                                        .clone(),
                                                                    folder.clone(),
                                                                    cx,
                                                                ),
                                                                cx,
                                                            );
                                                        });
                                                    }
                                                });
                                            });

                                        vec![cancel, submit]
                                    })
                            });
                        }
                    });
                }
                Err(err) => {
                    let _ = this.update_in(cx, |_this, window, cx| {
                        window.push_notification(
                            Notification::error(format!("{err}")).title("Upload cancelled"),
                            cx,
                        );
                    });
                }
                _ => {}
            };
        })
        .detach();
    }
}

impl create_folder_dialog::CreateFolderDialog for BrowseUi {
    fn create_folder(&mut self, name: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();
        let nav_state = self.current_nav.clone();

        let task = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::create_folder(
                        &token,
                        nav_state.space_id(),
                        nav_state.folder_id(),
                        name.as_str().to_owned(),
                    )
                    .await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.creating_folder = true;
                cx.notify();
            });

            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.creating_folder = false;

                match result {
                    Ok(_) => {
                        window.close_all_dialogs(cx);

                        this.fetch_folders(window, cx);
                    }
                    Err(err) => window.push_notification(
                        Notification::error(err.message).title("Failed to create folder"),
                        cx,
                    ),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn current_path(&self) -> SharedString {
        self.folder
            .as_ref()
            .map(|f| f.path.clone())
            .unwrap_or_default()
    }

    fn is_loading(&self) -> bool {
        self.creating_folder
    }
}

impl delete_dialog::DeleteDialog for BrowseUi {
    fn delete(
        &mut self,
        fs_id: delete_dialog::DeleteType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let nav_state = self.current_nav.clone();
        let inner = self.auth.read(cx).inner();

        let fs = fs_id.clone();
        let task = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| match fs {
                    delete_dialog::DeleteType::File(uuids) => {
                        let mut err_message = String::from("");

                        for (file_id, name) in uuids.into_iter() {
                            let result =
                                api::cloud::delete_file(&token, nav_state.space_id(), &file_id)
                                    .await;
                            if let Err(err) = result {
                                err_message.push_str(name.as_str());
                                err_message.push_str(": ");
                                err_message.push_str(&err.message);
                                err_message.push_str("\n");
                            }
                        }

                        if err_message.is_empty() {
                            Ok(())
                        } else {
                            Err(AppError::message(err_message))
                        }
                    }
                    delete_dialog::DeleteType::Folder((uuid, _)) => {
                        api::cloud::delete_folder(&token, nav_state.space_id(), &uuid)
                            .await
                            .map(|_| ())
                    }
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.deleting = true;
                cx.notify();
            });

            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.deleting = false;

                match result {
                    Ok(_) => {
                        window.close_all_dialogs(cx);

                        match fs_id {
                            delete_dialog::DeleteType::File(_) => {
                                this.nav.update(cx, |_nav, cx| {
                                    cx.emit(NavEvent::RefreshView(this.current_nav.clone()));
                                });
                            }
                            delete_dialog::DeleteType::Folder((folder_id, _)) => {
                                this.fetch_folders(window, cx);

                                this.nav.update(cx, |nav, cx| {
                                    nav.remove_folder_views(folder_id, cx);
                                    cx.notify();
                                });
                            }
                        };
                    }
                    Err(err) => window.push_notification(
                        Notification::error(err.message)
                            .title(format!("Failed to delete {}", fs_id.get_type())),
                        cx,
                    ),
                };

                cx.notify();
            });
        })
        .detach();
    }

    fn is_loading(&self) -> bool {
        self.deleting
    }
}

impl NavId for BrowseUi {
    fn id(&self) -> NavState {
        self.current_nav.clone()
    }
}

impl Render for BrowseUi {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .when(self.file_checked.len() > 0, |this| {
                        this.child(self.render_selections(cx))
                    })
                    .child(deferred(self.render_browse_status(cx)).with_priority(999))
                    .map(|this| {
                        if self.media_state.read(cx).view_list().is_empty() {
                            this.child(
                                div().p_2().child(
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
                                        .child(div().rounded_md().p_2().bg(cx.theme().muted).child(
                                            Icon::new(IconName::GalleryVerticalEnd).size_5(),
                                        ))
                                        .child(div().text_lg().child("No media"))
                                        .child(div().child("Upload files to access them anywhere."))
                                        .child(
                                            Button::new("upload")
                                                .primary()
                                                .icon(Icon::empty().path("icons/upload.svg"))
                                                .label("Upload")
                                                .disabled(self.loading_folders),
                                        ),
                                ),
                            )
                        } else {
                            this.child(self.render_file_list(window, cx))
                        }
                    }),
            )
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

/// UI functions
impl BrowseUi {
    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        Sidebar::new(Side::Left)
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::GalleryVerticalEnd))
                        .child(format!(
                            "{} items",
                            self.folders.len() + self.media_state.read(cx).view_list().len()
                        )),
                ),
            )
            .when(!self.media_state.read(cx).sections().is_empty(), |this| {
                this.child(self.render_sidebar_timelies(cx))
            })
            .child(
                SidebarGroup::new("Folders").child(SidebarMenu::new().when_else(
                    self.loading_folders,
                    |el| {
                        el.child(
                            SidebarMenuItem::new("Loading")
                                .active(false)
                                .suffix(_components::loading_icon(|icon| icon.size_4())),
                        )
                    },
                    |el| {
                        el.when_else(
                            self.folders.is_empty(),
                            |el| el.child(SidebarMenuItem::new("No folders")),
                            |el| {
                                el.children(self.folders.iter().map(|folder| {
                                    Self::render_sidebar_folder_item(folder.clone(), cx)
                                }))
                            },
                        )
                    },
                )),
            )
            .footer(
                div().flex().w_full().gap_2().child(
                    create_folder_dialog::trigger(cx.weak_entity())
                        .flex_1()
                        .small()
                        .disabled(self.loading_folders),
                ),
            )
    }

    fn render_sidebar_timelies(&self, cx: &Context<Self>) -> SidebarGroup<SidebarMenu> {
        let sections = self.media_state.read(cx).sections();
        let entity = cx.weak_entity();

        SidebarGroup::new("Dates").child(
            SidebarMenu::new().child(
                SidebarMenuItem::new("Times")
                    .icon(IconName::Calendar)
                    .suffix(
                        Button::new("td")
                            .icon(IconName::ChevronRight)
                            .small()
                            .ghost()
                            .on_click(move |_ev, _window, cx| {
                                cx.stop_propagation();
                            })
                            .dropdown_menu(move |menu, _window, _cx| {
                                let mut menu = menu.scrollable(true);

                                for (date, offset) in sections.iter().cloned() {
                                    let entity = entity.clone();

                                    menu = menu.menu_element(
                                        Box::new(EmptyAction),
                                        move |_window, _cx| {
                                            let entity = entity.clone();

                                            div()
                                                .id("")
                                                .flex()
                                                .gap_2()
                                                .items_center()
                                                .child(Icon::empty().path("icons/clock.svg"))
                                                .child(
                                                    div()
                                                        .child(
                                                            date.format("%a, %B %d, %Y")
                                                                .to_string(),
                                                        )
                                                        .text_sm(),
                                                )
                                                .on_click(move |_ev, _window, cx| {
                                                    let entity = entity.clone();

                                                    let _ = entity.update(cx, |this, cx| {
                                                        this.files_scroll_handle.set_offset(
                                                            Point::new(px(0.), offset.negate()),
                                                        );
                                                        cx.notify();
                                                    });
                                                })
                                        },
                                    );
                                }
                                menu
                            }),
                    ),
            ),
        )
    }

    fn render_sidebar_folder_item(folder: FolderResponse, cx: &Context<Self>) -> SidebarMenuItem {
        let folder_id = folder.id.clone();
        let folder_name = folder.name.clone();
        let entity = cx.weak_entity();

        SidebarMenuItem::new(&folder.name)
            .icon(Icon::new(IconName::Folder))
            .suffix(
                Button::new("")
                    .icon(IconName::EllipsisVertical)
                    .small()
                    .ghost()
                    .on_click(move |_ev, _window, cx| {
                        cx.stop_propagation();
                    })
                    .dropdown_menu(move |menu, _window, _cx| {
                        let folder_name = folder_name.clone();
                        let entity = entity.clone();

                        menu.menu_element(Box::new(EmptyAction), move |_window, cx| {
                            let folder_name = folder_name.clone();
                            let entity = entity.clone();

                            div()
                                .id("")
                                .flex()
                                .gap_2()
                                .items_center()
                                .text_color(cx.theme().danger)
                                .child(Icon::new(IconName::Delete).small())
                                .child(div().child("Delete folder").text_sm())
                                .on_click(move |_ev, window, cx| {
                                    let folder_name = folder_name.clone();
                                    let entity = entity.clone();

                                    window.open_dialog(cx, move |dialog, _window, cx| {
                                        delete_dialog::comp(
                                            dialog,
                                            entity.clone(),
                                            delete_dialog::DeleteType::Folder((
                                                folder_id.clone(),
                                                folder_name.clone(),
                                            )),
                                            cx,
                                        )
                                    });
                                })
                        })
                    }),
            )
            .on_click(cx.listener(move |this, _ev, window, cx| {
                this.nav.update(cx, |stack, cx| {
                    stack.push(
                        BrowseUi::view(
                            this.auth.clone(),
                            this.user_data.clone(),
                            this.nav.clone(),
                            this.upload_manager.clone(),
                            this.current_nav.clone().with_folder(folder_id),
                            window,
                            cx,
                        ),
                        cx,
                    );
                });
            }))
    }

    fn render_browse_status(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .bottom_0()
            .flex()
            .flex_shrink_0()
            .items_center()
            .bg(cx.theme().sidebar)
            .border_t_1()
            .border_color(cx.theme().sidebar_border)
            .px_2()
            .py_1p5()
            .w_full()
            .justify_between()
            .child(
                h_flex()
                    .id("browse_footer")
                    .w_56()
                    .overflow_x_scroll()
                    .track_scroll(&self.header_scroll_handle)
                    .text_sm()
                    .map(|this| {
                        if self.loading_folders {
                            this.child(loading_icon(|icon| icon.size_4()))
                        } else if let Some(folder) = self.folder.as_ref() {
                            this.child(folder.path.replace("/", " / "))
                        } else {
                            this.child("...")
                        }
                    }),
            )
            .child(
                Button::new("upload")
                    .primary()
                    .icon(Icon::empty().path("icons/upload.svg"))
                    .label("Upload")
                    .small()
                    .disabled(self.loading_folders)
                    .on_click(cx.listener(|this, _ev, window, cx| {
                        // window.prompt(level, message, detail, answers, cx)

                        this.open_upload(window, cx);
                    })),
            )
            .horizontal_scrollbar(&self.header_scroll_handle)
    }

    fn render_selections(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .bg(cx.theme().primary.opacity(0.2))
            .px_2()
            .py_1p5()
            .w_full()
            .justify_between()
            .child(
                h_flex()
                    .id("selection_header")
                    .gap_3()
                    .text_sm()
                    .child(format!("{} files selected", self.file_checked.len()))
                    .child(
                        Button::new("selection_clear")
                            .primary()
                            .icon(IconName::Close)
                            .label("Clear")
                            .small()
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                this.file_checked.clear();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Button::new("delete_files")
                    .danger()
                    .icon(IconName::Delete)
                    .label(format!("Delete {} files", self.file_checked.len()))
                    .small()
                    .disabled(self.loading_folders)
                    .on_click(cx.listener(|this, _ev, window, cx| {
                        let entity = cx.weak_entity();

                        this.media_state.read(cx).view_list();

                        window.open_dialog(cx, move |dialog, _window, cx| {
                            let ty = entity
                                .read_with(cx, |this, _cx| {
                                    this.file_checked
                                        .iter()
                                        .map(|(id, name)| (id.clone(), name.clone()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            delete_dialog::comp(
                                dialog,
                                entity.clone(),
                                delete_dialog::DeleteType::File(ty),
                                cx,
                            )
                        });
                    })),
            )
    }

    fn render_file_list(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("browse_ui")
            .size_full()
            .overflow_y_scroll()
            .child(
                div()
                    .id("files_scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .p_2()
                    .pb_12()
                    .child(
                        div().flex().flex_wrap().gap(MEDIA_GAP).children(
                            self.media_state
                                .read(cx)
                                .view_list()
                                .iter()
                                .cloned()
                                .enumerate()
                                .map(|(i, element_type)| match element_type {
                                    ElementType::File(file) => self.render_file_item(i, file, cx),
                                    ElementType::Section(date) => div()
                                        .h(SECTION_HEIGHT)
                                        .px_2()
                                        .flex()
                                        .items_center()
                                        .flex_grow()
                                        .w_full()
                                        .rounded_md()
                                        .font_semibold()
                                        .text_sm()
                                        .child(date.format("%a, %B %d, %Y").to_string()),
                                }),
                        ),
                    )
                    .child({
                        let this = cx.entity();
                        canvas(
                            move |_b, _w, _c| {},
                            move |_b, _d, _w, cx| {
                                this.update(cx, |this, cx| {
                                    let size = this.files_scroll_handle.bounds().size;
                                    let emitted = this.render_bounds.update(cx, |bounds, cx| {
                                        bounds
                                            .w_event(size.width)
                                            .map(|ev| {
                                                cx.emit(ev);
                                                true
                                            })
                                            .unwrap_or(false)
                                    });
                                    if !emitted {
                                        Self::update_visible_state(this, cx);
                                    }
                                });
                            },
                        )
                    })
                    .track_scroll(&self.files_scroll_handle),
            )
            .vertical_scrollbar(&self.files_scroll_handle)
    }

    fn render_file_item(&self, i: usize, file: Arc<FileMetaReponse>, cx: &Context<Self>) -> Div {
        let _file = file.clone();
        let this = cx.entity();

        if self.visible_item_range.contains(&i) {
            div()
                .h(MEDIA_HEIGHT)
                .w(px(file.width as f32))
                .rounded_md()
                .relative()
                .group(SharedString::new(file.id.to_string()))
                .flex_shrink_0()
                .map(|this| {
                    if let Some(MediaAssetState::Loaded { thumbnail_path, .. }) =
                        self.media_state.read(cx).asset(&file.id).cloned()
                    {
                        this.child(
                            img(ImageSource::Resource(Resource::Path(thumbnail_path.into())))
                                .object_fit(ObjectFit::Cover)
                                .flex()
                                .items_center()
                                .justify_center()
                                .id(SharedString::new(format!("{i}")))
                                .absolute()
                                .inset_0()
                                .h(MEDIA_HEIGHT)
                                .w(px(file.width as f32))
                                .rounded_md()
                                .overflow_hidden()
                                .with_loading(|| {
                                    loading_icon(|icon| icon.size_4()).into_any_element()
                                })
                                .with_fallback(|| {
                                    Icon::new(IconName::TriangleAlert).into_any_element()
                                })
                                .border_1()
                                .border_color(cx.theme().sidebar_border)
                                .when(self.file_checked.contains_key(&file.id), |this| {
                                    this.border_2().border_color(cx.theme().primary)
                                })
                                .on_click(cx.listener(move |this, _ev, window, cx| {
                                    Self::open_media(this, i, window, cx);
                                })),
                        )
                        .when(
                            matches!(file.media_type, MediaType::Video),
                            |this| {
                                this.child(
                                    div()
                                        .absolute()
                                        .inset_0()
                                        .size_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .p_2()
                                                .rounded_full()
                                                .bg(black().opacity(0.3))
                                                .text_color(white())
                                                .child(Icon::empty().path("icons/play.svg")),
                                        ),
                                )
                            },
                        )
                    } else {
                        this.bg(cx.theme().muted)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(loading_icon(|icon| icon.size_4()))
                    }
                })
                .child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .bg(black().opacity(0.3))
                        .rounded_b_md()
                        .text_color(white())
                        .text_sm()
                        .px_2()
                        .py_1()
                        .opacity(0.)
                        .group_hover(SharedString::new(file.id.to_string()), |el| {
                            el.opacity(100.)
                        })
                        .truncate()
                        .child(file.file_name.clone()),
                )
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .flex()
                        .w_full()
                        .items_center()
                        .justify_end()
                        .map(|this| {
                            if !self.file_checked.contains_key(&file.id) {
                                this.opacity(0.)
                                    .group_hover(SharedString::new(file.id.to_string()), |el| {
                                        el.opacity(100.)
                                    })
                            } else {
                                this
                            }
                        })
                        .child(
                            Checkbox::new(SharedString::new(format!("checkbox_{}", file.id)))
                                .p_2()
                                .checked(self.file_checked.contains_key(&file.id))
                                .on_click(cx.listener(move |this, checked, _, cx| {
                                    cx.stop_propagation();

                                    if *checked {
                                        this.file_checked
                                            .insert(file.id.clone(), file.file_name.clone());
                                    } else {
                                        this.file_checked.remove(&file.id);
                                    }
                                    cx.notify();
                                })),
                        ),
                )
                .on_children_prepainted(move |_b, window, cx| {
                    this.update(cx, |this, cx| {
                        this.fetch_image_urls(window, cx, _file.clone());
                        cx.notify();
                    });
                })
        } else {
            div()
                .h(MEDIA_HEIGHT)
                .w(px(file.width as f32))
                .rounded_md()
                .bg(cx.theme().sidebar)
                .child(file.file_name.clone())
        }
    }
}
