use std::{collections::HashSet, ops::Range};

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Side, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    h_flex,
    menu::DropdownMenu,
    notification::Notification,
    scroll::ScrollableElement,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};
use uuid::Uuid;

use crate::{
    auth::AuthState,
    ctx::{FetchMedia, MediaData, UrlState, UserData},
    nav::{NavEvent, NavId, NavState, Navigation},
    rt,
    ui::{
        _components::{
            self, MEDIA_HEIGHT, RenderBounds, create_folder_dialog, delete_dialog, loading_icon,
        },
        header::EmptyAction,
        home::media::MediaUi,
    },
    util::MapAsync,
    web::api::{
        self,
        models::cloud::{MediaType, res::FolderResponse},
    },
};

pub struct BrowseUi {
    auth: AuthState,
    user_data: Entity<UserData>,
    nav: Navigation,
    render_bounds: Entity<RenderBounds>,
    image_cache: Entity<RetainAllImageCache>,

    current_nav: NavState,
    folder: Option<FolderResponse>,
    folders: Vec<FolderResponse>,
    media_data: Entity<MediaData>,
    file_checked: HashSet<Uuid>,

    item_rows: Vec<Range<usize>>,
    visible_rows: usize,
    visible_item_range: Range<usize>,
    scroll_offset: Pixels,
    files_scroll_handle: ScrollHandle,
    header_scroll_handle: ScrollHandle,

    loading_folders: bool,
    loading_files: bool,
    creating_folder: bool,
    deleting_folder: bool,
    _subscriptions: Vec<Subscription>,
}

impl BrowseUi {
    fn new(
        auth: AuthState,
        user_data: Entity<UserData>,
        nav: Navigation,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let image_cache = RetainAllImageCache::new(cx);

        let media_data = cx.new(|_cx| MediaData::new());
        let render_bounds = cx.new(|_cx| RenderBounds::new());

        let size_sub = cx.subscribe_in(
            &render_bounds,
            window,
            |this, _entity, _event, _window, cx| {
                Self::compute_visible_grid(this, cx);
                this.scroll_offset = px(-1.);
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

        let media_sub = cx.subscribe_in(&media_data, window, |this, _entity, event, window, cx| {
            let FetchMedia { file_id, index } = event;
            this.fetch_image_urls(window, cx, *index, file_id.clone());
        });

        Self {
            auth,
            user_data,
            nav,
            render_bounds,
            image_cache,
            current_nav,
            folder: None,
            folders: Vec::new(),
            media_data,
            file_checked: HashSet::new(),
            item_rows: Vec::new(),
            visible_rows: 0,
            visible_item_range: 0..0,
            scroll_offset: px(-1.),
            files_scroll_handle: ScrollHandle::new(),
            header_scroll_handle: ScrollHandle::new(),
            loading_folders: false,
            loading_files: false,
            creating_folder: false,
            deleting_folder: false,
            _subscriptions: vec![size_sub, nav_sub, media_sub],
        }
    }

    pub fn view(
        auth: AuthState,
        user_data: Entity<UserData>,
        nav: Navigation,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let mut entity = Self::new(auth, user_data, nav, current_nav, window, cx);
            entity.fetch_folders(window, cx);
            entity.fetch_files(window, cx);
            entity
        })
    }

    fn reset_state(&mut self, cx: &mut Context<Self>) {
        self.folders.clear();
        self.media_data.update(cx, |md, cx| {
            md.reset();
            cx.notify();
        });
        self.file_checked.clear();
        self.item_rows.clear();
        self.visible_rows = 0;
        self.visible_item_range = 0..0;
        self.scroll_offset = px(-1.);
    }

    fn compute_visible_grid(this: &mut Self, cx: &mut Context<Self>) {
        let files = this.media_data.read(cx).data();
        if files.is_empty() {
            return;
        }

        let bounds = this.render_bounds.read(cx);

        let mut used_w = 16f32; // px-4
        let mut start = 0;
        let mut items_in_row = 0;
        this.item_rows = Vec::new();

        for (file, _) in files.iter() {
            let fw = file.width as f32;

            if px(fw + used_w + 1.) > bounds.width {
                this.item_rows.push(start..(start + items_in_row));

                start += items_in_row;
                items_in_row = 1;
                used_w = 16. + fw + 1.;
            } else {
                used_w += fw + 1.; // gap-1
                items_in_row += 1;
            }
        }
        this.item_rows.push(start..files.len());

        this.visible_rows = this
            .item_rows
            .len()
            .min(((bounds.height / MEDIA_HEIGHT).ceil()) as usize + 3);

        let start = this.item_rows.get(0).map(|r| r.start).unwrap_or(0);
        let end = this
            .item_rows
            .get(this.visible_rows)
            .map(|r| r.end)
            .unwrap_or(0);

        this.visible_item_range = start..end;

        cx.notify();
    }

    fn update_visible_state(this: &mut Self, cx: &mut Context<Self>) {
        let offset = this.files_scroll_handle.offset().y;
        if this.scroll_offset == offset {
            return;
        }

        this.scroll_offset = offset + px(34.) /* title bar offset */;

        let media_height = MEDIA_HEIGHT + px(4.) /* vertical gap */;

        let start_index = (this.scroll_offset.abs() / media_height).floor() as usize;
        let start_index = start_index.min(this.item_rows.len() - this.visible_rows);
        let end_index = (start_index + this.visible_rows).min(this.item_rows.len());

        // println!(
        //     "offset: {} [{}]{} - start[{}]: {:?} - end[{}]: {:?}",
        //     offset,
        //     end_index - start_index,
        //     this.visible_rows,
        //     start_index,
        //     this.item_rows.get(start_index),
        //     end_index,
        //     this.item_rows.get(end_index)
        // );

        let start = this
            .item_rows
            .get(start_index)
            .map(|r| r.start)
            .unwrap_or(0);

        let end = this
            .item_rows
            .get(end_index.checked_sub(1).unwrap_or(0))
            .map(|r| r.end)
            .unwrap_or(0);

        this.visible_item_range = start..end;

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
                        Notification::error(err.message)
                            .title("Failed to fetch folders")
                            .autohide(false),
                        cx,
                    ),
                };

                match folder {
                    Ok(folder) => this.folder = Some(folder),
                    Err(err) => window.push_notification(
                        Notification::error(err.message)
                            .title("Failed to get current folder")
                            .autohide(false),
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
                        this.media_data.update(cx, |md, cx| {
                            md.set_files(files);
                            cx.notify();
                        });

                        Self::compute_visible_grid(this, cx);

                        this.scroll_offset = px(-1.); // invalidate
                    }
                    Err(err) => window.push_notification(
                        Notification::error(err.message)
                            .title("Failed to get files")
                            .autohide(false),
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
        index: usize,
        file_id: Uuid,
    ) {
        let nav_state = self.current_nav.clone();

        let queued_or_loaded = self.media_data.update(cx, |md, cx| {
            if let Some((_, state)) = md.data_mut(index) {
                match state {
                    UrlState::Queued | UrlState::Loaded(_) => {
                        return true;
                    }
                    _ => (),
                };

                *state = UrlState::Queued;
                cx.notify();

                return false;
            }
            true
        });

        if queued_or_loaded {
            return;
        }

        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .read_with(cx, |this, cx| this.auth.read(cx).inner())
                .unwrap();

            let urls = rt::spawn(cx, async move {
                inner
                    .get_token()
                    .await
                    .map_async(async move |token| {
                        api::cloud::get_stream_urls(&token, nav_state.space_id(), &file_id).await
                    })
                    .await
                    .unwrap()
            })
            .unwrap()
            .await;

            let _ = this.update_in(cx, |this, window, cx| {
                match urls {
                    Ok(urls) => {
                        this.media_data.update(cx, |md, cx| {
                            md.update_url(index, UrlState::Loaded(urls));
                            cx.notify();
                        });
                    }
                    Err(err) => {
                        this.media_data.update(cx, |md, cx| {
                            md.update_url(index, UrlState::Error);
                            cx.notify();
                        });
                        window.push_notification(
                            Notification::error(err.message).autohide(false),
                            cx,
                        );
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
                    this.media_data.clone(),
                    index,
                    this.image_cache.clone(),
                    window,
                    cx,
                ),
                cx,
            );
            cx.notify();
        });
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

    fn current_path(&self) -> String {
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
                    delete_dialog::DeleteType::File(_uuid) => {
                        todo!()
                    }
                    delete_dialog::DeleteType::Folder(uuid) => {
                        api::cloud::delete_folder(&token, nav_state.space_id(), &uuid).await
                    }
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.deleting_folder = true;
                cx.notify();
            });

            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.deleting_folder = false;

                match result {
                    Ok(_) => {
                        window.close_all_dialogs(cx);

                        match fs_id {
                            delete_dialog::DeleteType::File(_) => {
                                this.nav.update(cx, |_nav, cx| {
                                    cx.emit(NavEvent::RefreshView(this.current_nav.clone()));
                                });
                            }
                            delete_dialog::DeleteType::Folder(folder_id) => {
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
        self.deleting_folder
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
                    // .child(self.render_folder_cards(cx))
                    .map(|this| {
                        if self.media_data.read(cx).data().is_empty() {
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
                        .child(format!("{} items", self.folders.len() + self.media_data.read(cx).data().len())),
                ),
            )
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
                                    let folder_id = folder.id.clone();
                                    let folder_path = folder.path.clone();
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
                                                    let folder_path = folder_path.clone();
                                                    let entity = entity.clone();

                                                    menu.menu_element(
                                                        Box::new(EmptyAction),
                                                        move |_window, cx| {
                                                            let folder_path = folder_path.clone();
                                                            let entity = entity.clone();

                                                            div()
                                                                .id("")
                                                                .flex()
                                                                .gap_2()
                                                                .items_center()
                                                                .text_color(cx.theme().danger)
                                                                .child(
                                                                    Icon::new(IconName::Delete)
                                                                        .small(),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .child("Delete folder")
                                                                        .text_sm(),
                                                                )
                                                                .on_click(move |_ev, window, cx| {
                                                                    let folder_path = folder_path.clone();
                                                                    let entity = entity.clone();

                                                                    window.open_dialog(cx, move |dialog, _window, cx| {
                                                                        delete_dialog::comp(dialog, entity.clone(), delete_dialog::DeleteType::Folder(folder_id.clone()), folder_path.clone(), cx)
                                                                    });
                                                                })
                                                        },
                                                    )
                                                }),
                                        )
                                        .on_click(cx.listener(move |this, _ev, window, cx| {
                                            this.nav.update(cx, |stack, cx| {
                                                stack.push(
                                                    BrowseUi::view(
                                                        this.auth.clone(),
                                                        this.user_data.clone(),
                                                        this.nav.clone(),
                                                        this.current_nav
                                                            .clone()
                                                            .with_folder(folder_id),
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
            .footer(
                create_folder_dialog::trigger(cx.weak_entity())
                    .w_full()
                    .small()
                    .disabled(self.loading_folders),
            )
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
                    .disabled(self.loading_folders),
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
                    .disabled(self.loading_folders),
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
                    .child(
                        div().flex().flex_wrap().gap_1().pb_12().children(
                            self.media_data
                                .read(cx)
                                .data()
                                .iter()
                                .cloned()
                                .enumerate()
                                .map(|(i, (file, url_state))| {
                                    let _file = file.clone();
                                    // let file_name = file.file_name.clone();

                                    let this = cx.entity();
                                    // let entity = cx.weak_entity();
                                    if self.visible_item_range.contains(&i) {
                                        div()
                                            .h(MEDIA_HEIGHT)
                                            .w(px(file.width as f32))
                                            .rounded_md()
                                            .relative()
                                            .group(SharedString::new(file.id.to_string()))
                                            .flex_shrink_0()
                                            .map(|this| {
                                                if let UrlState::Loaded(urls) = url_state {
                                                    this.child(
                                                        img(ImageSource::Resource(Resource::Uri(
                                                            SharedUri::from(&urls.thumbnail_stream),
                                                        )))
                                                        .image_cache(&self.image_cache)
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
                                                            loading_icon(|icon| icon.size_4())
                                                                .into_any_element()
                                                        })
                                                        .border_1()
                                                        .border_color(cx.theme().sidebar_border)
                                                        .when(
                                                            self.file_checked.contains(&file.id),
                                                            |this| {
                                                                this.border_2().border_color(
                                                                    cx.theme().primary,
                                                                )
                                                            },
                                                        )
                                                        .on_click(cx.listener(
                                                            move |this, _ev, window, cx| {
                                                                Self::open_media(
                                                                    this, i, window, cx,
                                                                );
                                                            },
                                                        )),
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
                                                                        .child(Icon::empty().path(
                                                                            "icons/play.svg",
                                                                        )),
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
                                                    .group_hover(
                                                        SharedString::new(file.id.to_string()),
                                                        |el| el.opacity(100.),
                                                    )
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
                                                        if !self.file_checked.contains(&file.id) {
                                                            this.opacity(0.).group_hover(
                                                                SharedString::new(
                                                                    file.id.to_string(),
                                                                ),
                                                                |el| el.opacity(100.),
                                                            )
                                                        } else {
                                                            this
                                                        }
                                                    })
                                                    .child(
                                                        Checkbox::new(SharedString::new(format!(
                                                            "checkbox_{}",
                                                            file.id
                                                        )))
                                                        .p_2()
                                                        .checked(
                                                            self.file_checked.contains(&file.id),
                                                        )
                                                        .on_click(cx.listener(
                                                            move |this, checked, _, cx| {
                                                                cx.stop_propagation();

                                                                if *checked {
                                                                    this.file_checked
                                                                        .insert(file.id.clone());
                                                                } else {
                                                                    this.file_checked
                                                                        .remove(&file.id);
                                                                }
                                                                cx.notify();
                                                            },
                                                        )),
                                                    ),
                                            )
                                            .on_children_prepainted(move |_b, window, cx| {
                                                this.update(cx, |this, cx| {
                                                    this.fetch_image_urls(
                                                        window,
                                                        cx,
                                                        i,
                                                        _file.id.clone(),
                                                    );
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
}
