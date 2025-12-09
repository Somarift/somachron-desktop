use std::ops::Range;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Side, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    notification::Notification,
    scroll::{ScrollableElement, ScrollbarAxis},
    sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use uuid::Uuid;

use crate::{
    auth::AuthState,
    ctx::UserState,
    nav::{NavEvent, NavId, NavState, Navigation},
    rt,
    ui::_components::{self, MEDIA_HEIGHT, RenderBounds, loading_icon},
    util::MapAsync,
    web::api::{
        self,
        models::cloud::res::{FileMetaReponse, FolderResponse, StreamedUrlsResponse},
    },
};

#[derive(Debug, Clone)]
enum MediaState {
    Idle,
    Queued,
    Loaded(StreamedUrlsResponse),
    Error,
}

pub struct BrowseUi {
    auth: AuthState,
    user_state: UserState,
    nav: Navigation,
    render_bounds: Entity<RenderBounds>,
    image_cache: Entity<RetainAllImageCache>,

    current_nav: NavState,
    folder: Option<FolderResponse>,
    folders: Vec<FolderResponse>,
    files: Vec<FileMetaReponse>,
    media_states: Vec<MediaState>,

    item_rows: Vec<Range<usize>>,
    visible_rows: usize,
    visible_item_range: Range<usize>,
    scroll_offset: Pixels,
    files_scroll_handle: ScrollHandle,
    header_scroll_handle: ScrollHandle,

    loading: bool,
    _subscriptions: Vec<Subscription>,
}

impl BrowseUi {
    fn new(
        auth: AuthState,
        user_state: UserState,
        nav: Navigation,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let image_cache = RetainAllImageCache::new(cx);

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
            if let NavEvent::Refresh = event {
                this.reset_state();
                this.fetch_fs(window, cx);
            }
        });

        Self {
            auth,
            user_state,
            nav,
            render_bounds,
            image_cache,
            current_nav,
            folder: None,
            folders: Vec::new(),
            files: Vec::new(),
            media_states: Vec::new(),
            item_rows: Vec::new(),
            visible_rows: 0,
            visible_item_range: 0..0,
            scroll_offset: px(-1.),
            files_scroll_handle: ScrollHandle::new(),
            header_scroll_handle: ScrollHandle::new(),
            loading: false,
            _subscriptions: vec![size_sub, nav_sub],
        }
    }

    pub fn view(
        auth: AuthState,
        user_state: UserState,
        nav: Navigation,
        current_nav: NavState,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let mut entity = Self::new(auth, user_state, nav, current_nav, window, cx);
            entity.fetch_fs(window, cx);
            entity
        })
    }

    fn reset_state(&mut self) {
        self.folders.clear();
        self.files.clear();
        self.media_states.clear();
        self.item_rows.clear();
        self.visible_rows = 0;
        self.visible_item_range = 0..0;
        self.scroll_offset = px(-1.);
    }

    fn compute_visible_grid(this: &mut Self, cx: &mut Context<Self>) {
        if this.files.is_empty() {
            return;
        }

        let bounds = this.render_bounds.read(cx);

        let mut used_w = 16f32; // px-4
        let mut start = 0;
        let mut items_in_row = 0;
        this.item_rows = Vec::new();

        for file in this.files.iter() {
            let fw = file.width as f32;

            if px(fw + used_w + 2.) > bounds.width {
                this.item_rows.push(start..(start + items_in_row));

                start += items_in_row;
                items_in_row = 1;
                used_w = 16. + fw + 2.;
            } else {
                used_w += fw + 2.; // gap-2
                items_in_row += 1;
            }
        }
        this.item_rows.push(start..this.files.len());

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

        this.scroll_offset = offset;

        let start_index = (this.scroll_offset.abs() / (MEDIA_HEIGHT * 2.)).floor() as usize;
        let start_index = start_index.min(this.item_rows.len() - this.visible_rows);
        let start_offset = (this.scroll_offset.abs() / MEDIA_HEIGHT) as usize;
        let end_index = (start_offset + this.visible_rows).min(this.item_rows.len());

        // println!(
        //     "offset: {} [{}] - start[{}]: {:?} - end[{}]: {:?}",
        //     offset,
        //     end_index - start_index,
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

    fn fetch_fs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.current_nav.clone();
        let space_id = state.space_id.clone();
        let folder_id = state.folder_id.clone();

        let inner = self.auth.read(cx).inner();

        let _inner = inner.clone();
        let folders = rt::spawn(cx, async move {
            _inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::list_folders(&token, &space_id.clone(), &folder_id.clone()).await
                })
                .await
        });

        let _inner = inner.clone();
        let files = rt::spawn(cx, async move {
            _inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::list_files(
                        &token,
                        &state.space_id.clone(),
                        &state.folder_id.clone(),
                    )
                    .await
                })
                .await
        });

        let folder = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::get_folder(
                        &token,
                        &state.space_id.clone(),
                        &state.folder_id.clone(),
                    )
                    .await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading = true;
                cx.notify();
            });

            let folders = folders.await.flatten();
            let folder = folder.await.flatten();
            let files = files.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading = false;
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

                match files {
                    Ok(files) => {
                        this.files = files;
                        this.media_states = this.files.iter().map(|_| MediaState::Idle).collect();
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

                match folder {
                    Ok(folder) => this.folder = Some(folder),
                    Err(err) => window.push_notification(
                        Notification::error(err.message)
                            .title("Failed to get current folder")
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
        this: &mut Self,
        window: &mut Window,
        cx: &mut Context<Self>,
        index: usize,
        file_id: Uuid,
    ) {
        let nav_state = this.current_nav.clone();

        let Some(state) = this.media_states.get_mut(index) else {
            return;
        };

        match state {
            MediaState::Queued | MediaState::Loaded(_) => {
                return;
            }
            _ => (),
        };

        *state = MediaState::Queued;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .read_with(cx, |this, cx| this.auth.read(cx).inner())
                .unwrap();

            let urls = rt::spawn(cx, async move {
                inner
                    .get_token()
                    .await
                    .map_async(async move |token| {
                        api::cloud::get_stream_urls(&token, &nav_state.space_id, &file_id).await
                    })
                    .await
                    .unwrap()
            })
            .unwrap()
            .await;

            let _ = this.update_in(cx, |this, window, cx| {
                match urls {
                    Ok(urls) => {
                        this.media_states
                            .get_mut(index)
                            .map(|state| *state = MediaState::Loaded(urls));
                    }
                    Err(err) => {
                        this.media_states
                            .get_mut(index)
                            .map(|state| *state = MediaState::Error);
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
                    .child(
                        deferred(
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
                                        .id("browse_header")
                                        .w_56()
                                        .overflow_x_scroll()
                                        .track_scroll(&self.header_scroll_handle)
                                        .text_sm()
                                        .map(|this| {
                                            if self.loading {
                                                this.child(loading_icon(|icon| icon.size_4()))
                                            } else if let Some(folder) = self.folder.as_ref() {
                                                this.child(folder.path.replace("/", " / "))
                                            } else {
                                                this.child("...")
                                            }
                                        }),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("members")
                                                .icon(Icon::empty().path("icons/users.svg"))
                                                .small()
                                                .ghost()
                                                .border_1()
                                                .border_color(cx.theme().sidebar_border),
                                        )
                                        .child(
                                            Button::new("upload")
                                                .primary()
                                                .icon(Icon::empty().path("icons/upload.svg"))
                                                .label("Upload")
                                                .small(),
                                        ),
                                )
                                .horizontal_scrollbar(&self.header_scroll_handle),
                        )
                        .with_priority(999),
                    )
                    // .child(self.render_folder_cards(cx))
                    .child(
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
                                    .child(self.render_file_list(window, cx))
                                    .child({
                                        let this = cx.entity();
                                        canvas(
                                            move |_b, _w, _c| {},
                                            move |_b, _d, _w, cx| {
                                                this.update(cx, |this, cx| {
                                                    let size =
                                                        this.files_scroll_handle.bounds().size;
                                                    let emitted = this.render_bounds.update(
                                                        cx,
                                                        |bounds, cx| {
                                                            bounds
                                                                .w_event(size.width)
                                                                .map(|ev| {
                                                                    cx.emit(ev);
                                                                    true
                                                                })
                                                                .unwrap_or(false)
                                                        },
                                                    );
                                                    if !emitted {
                                                        Self::update_visible_state(this, cx);
                                                    }
                                                });
                                            },
                                        )
                                    })
                                    .track_scroll(&self.files_scroll_handle),
                            )
                            .vertical_scrollbar(&self.files_scroll_handle),
                    ),
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
            // .border_width(0.)
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::GalleryVerticalEnd))
                        .child(format!("{} items", self.folders.len() + self.files.len())),
                ),
            )
            .child(
                SidebarGroup::new("Folders").child(SidebarMenu::new().when_else(
                    self.loading,
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

                                    SidebarMenuItem::new(&folder.name)
                                        .icon(Icon::new(IconName::Folder))
                                        .on_click(cx.listener(move |this, _ev, window, cx| {
                                            this.nav.update(cx, |stack, cx| {
                                                stack.push(
                                                    BrowseUi::view(
                                                        this.auth.clone(),
                                                        this.user_state.clone(),
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
                Button::new("create_folder")
                    .primary()
                    .w_full()
                    .icon(Icon::empty().path("icons/folder-plus.svg"))
                    .label("Create folder")
                    .small(),
            )
    }

    fn render_folder_cards(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .flex_auto()
            .gap_2()
            .children(self.folders.iter().cloned().map(|folder| {
                div()
                    .flex_auto()
                    .id(SharedString::new(folder.id.to_string()))
                    .on_click(cx.listener(move |this, _ev, window, cx| {
                        let folder_id = &folder.id;
                        this.nav.update(cx, |stack, cx| {
                            stack.push(
                                BrowseUi::view(
                                    this.auth.clone(),
                                    this.user_state.clone(),
                                    this.nav.clone(),
                                    NavState::new(this.current_nav.space_id, folder_id.clone()),
                                    window,
                                    cx,
                                ),
                                cx,
                            );
                            cx.notify();
                        });
                    }))
                    .border_1()
                    .rounded_lg()
                    .p_4()
                    .bg(cx.theme().accent)
                    .child(
                        div()
                            .flex()
                            .justify_start()
                            .items_center()
                            .gap_4()
                            .child(Icon::new(IconName::Folder).size_4())
                            .child(div().text_sm().flex_wrap().font_medium().child(folder.name)),
                    )
            }))
    }

    fn render_file_list(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().flex().flex_wrap().gap_2().pb_12().children(
            self.files.iter().cloned().enumerate().map(|(i, file)| {
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
                            if let Some(MediaState::Loaded(urls)) =
                                self.media_states.get(i).cloned()
                            {
                                this.child(
                                    img(ImageSource::Resource(Resource::Uri(SharedUri::from(
                                        urls.thumbnail_stream,
                                    ))))
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
                                        loading_icon(|icon| icon.size_4()).into_any_element()
                                    }),
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
                        .on_children_prepainted(move |_b, window, cx| {
                            this.update(cx, |this, cx| {
                                Self::fetch_image_urls(this, window, cx, i, file.id.clone());
                                cx.notify();
                            });
                        })
                } else {
                    div()
                        .h(MEDIA_HEIGHT)
                        .w(px(file.width as f32))
                        .rounded_md()
                        .bg(cx.theme().sidebar)
                        .child(file.file_name)
                }
            }),
        )
    }
}
