use std::ops::Range;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, StyledExt, WindowExt, notification::Notification,
    scroll::ScrollbarAxis,
};

use crate::{
    auth::Auth,
    ui::_components::{self, MEDIA_HEIGHT, NavEvent, NavStack, NavState, RenderBounds},
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
    auth: Entity<Auth>,
    nav_stack: Entity<NavStack>,
    render_bounds: Entity<RenderBounds>,

    current_nav: Option<NavState>,
    folders: Vec<FolderResponse>,
    files: Vec<FileMetaReponse>,
    media_states: Vec<MediaState>,

    item_rows: Vec<Range<usize>>,
    visible_rows: usize,
    visible_item_range: Range<usize>,
    scroll_offset: Pixels,
    scroll_handle: ScrollHandle,

    loading: bool,
    _subscriptions: Vec<Subscription>,
}

impl BrowseUi {
    fn new(
        auth: Entity<Auth>,
        nav_stack: Entity<NavStack>,
        render_bounds: Entity<RenderBounds>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let size_sub = cx.subscribe_in(
            &render_bounds,
            window,
            |this, _entity, _event, _window, cx| {
                Self::compute_visible_grid(this, cx);
                this.scroll_offset = px(0.);
                Self::update_visible_state(this, cx);
            },
        );

        let nav_sub = cx.subscribe_in(&nav_stack, window, |this, _entity, event, window, cx| {
            this.reset_state();

            this.nav_stack.update(cx, |stack, cx| {
                stack.on_event(event);
                this.current_nav = stack.top();
                cx.notify();
            });

            this.fetch_fs(window, cx);
        });

        Self {
            auth,
            nav_stack,
            render_bounds,
            current_nav: None,
            folders: Vec::new(),
            files: Vec::new(),
            media_states: Vec::new(),
            item_rows: Vec::new(),
            visible_rows: 0,
            visible_item_range: 0..0,
            scroll_offset: px(0.),
            scroll_handle: ScrollHandle::new(),
            loading: false,
            _subscriptions: vec![nav_sub, size_sub],
        }
    }

    pub fn view(
        auth: Entity<Auth>,
        nav_stack: Entity<NavStack>,
        render_bounds: Entity<RenderBounds>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, nav_stack, render_bounds, window, cx))
    }

    fn reset_state(&mut self) {
        self.folders.clear();
        self.files.clear();
        self.media_states.clear();
        self.item_rows.clear();
        self.scroll_offset = px(0.);
        self.visible_rows = 0;
        self.visible_item_range = 0..0;
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

        // for range in this.item_rows.iter().take(10) {
        //     range.clone().for_each(|i| {
        //         print!("{} -- ", this.files.get(i).unwrap().file_name);
        //     });
        //     println!();
        // }

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
        let offset = this.scroll_handle.bounds().origin.y;
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
        let Some(state) = self.current_nav.as_ref().cloned() else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .update(cx, |this, cx| {
                    this.loading = true;
                    cx.notify();

                    this.auth.read(cx).inner()
                })
                .unwrap();

            let (folders, files) = cx
                .background_executor()
                .spawn(async move {
                    inner
                        .get_token()
                        .await
                        .map_async(async move |token| {
                            Ok(futures::join!(
                                api::cloud::list_folders(&token, &state.space_id, &state.folder_id),
                                api::cloud::list_files(&token, &state.space_id, &state.folder_id)
                            ))
                        })
                        .await
                        .unwrap()
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.loading = false;
                match folders {
                    Ok(folders) => {
                        this.folders = folders;
                    }
                    Err(err) => window
                        .push_notification(Notification::error(err.message).autohide(true), cx),
                };

                match files {
                    Ok(files) => {
                        this.files = files;
                        this.media_states = this.files.iter().map(|_| MediaState::Idle).collect();
                        Self::compute_visible_grid(this, cx);

                        this.scroll_offset = px(0.); // invalidate
                    }
                    Err(err) => window
                        .push_notification(Notification::error(err.message).autohide(true), cx),
                };
                cx.notify();
            })
            .unwrap();
        })
        .detach();
    }

    fn fetch_image_urls(
        this: &mut Self,
        window: &mut Window,
        cx: &mut Context<Self>,
        index: usize,
        file_id: String,
    ) {
        let Some(nav_state) = this.current_nav.as_ref().cloned() else {
            return;
        };

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

            let urls = cx
                .background_spawn(async move {
                    inner
                        .get_token()
                        .await
                        .map_async(async move |token| {
                            api::cloud::get_stream_urls(&token, &nav_state.space_id, &file_id).await
                        })
                        .await
                })
                .await;

            this.update_in(cx, |this, window, cx| {
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
                        window
                            .push_notification(Notification::error(err.message).autohide(true), cx);
                    }
                };
                cx.notify();
            })
            .unwrap();
        })
        .detach();
    }
}

impl Render for BrowseUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("browse_ui")
            .size_full()
            .scrollable(ScrollbarAxis::Vertical)
            .p_4()
            .when(self.loading, |el| {
                el.child(_components::loading_icon(|icon| icon.size_5()))
            })
            .when_some(self.current_nav.clone(), |el, state| {
                el.child(state.space_id)
                    .child(state.folder_id)
                    .child(self.render_folder_cards(cx))
                    .child(self.render_file_list(cx))
                    .child({
                        let this = cx.entity();
                        canvas(
                            move |_b, _w, _c| {},
                            move |_b, _d, _w, cx| {
                                this.update(cx, |this, cx| {
                                    let size = this.scroll_handle.bounds().size;
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
            })
            .track_scroll(&self.scroll_handle)
    }
}

/// UI functions
impl BrowseUi {
    fn render_folder_cards(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .flex_auto()
            .gap_2()
            .children(self.folders.iter().cloned().map(|folder| {
                div()
                    .flex_auto()
                    .id(SharedString::new(folder.id.clone()))
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.nav_stack.update(cx, |_stack, cx| {
                            cx.emit(NavEvent::Push(NavState::new(
                                this.current_nav.as_ref().unwrap().space_id.clone(),
                                folder.id.clone(),
                            )));
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

    fn render_file_list(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(self.files.iter().cloned().enumerate().map(|(i, file)| {
                let this = cx.entity();
                if self.visible_item_range.contains(&i) {
                    div()
                        .h(MEDIA_HEIGHT)
                        .w(px(file.width as f32))
                        .rounded_md()
                        .relative()
                        .group(SharedString::new(file.id.as_str()))
                        .bg(cx.theme().primary)
                        .child(
                            div()
                                .absolute()
                                .h_full()
                                .w(px(file.width as f32))
                                .rounded_lg()
                                .child(
                                    if let Some(MediaState::Loaded(urls)) =
                                        self.media_states.get(i).cloned()
                                    {
                                        div().child(
                                            img(urls.thumbnail_stream)
                                                .absolute()
                                                .inset_0()
                                                .h(MEDIA_HEIGHT)
                                                .w(px(file.width as f32))
                                                .object_fit(ObjectFit::Cover)
                                                .rounded_lg(),
                                        )
                                    } else {
                                        div().rounded_lg()
                                    },
                                ),
                        )
                        .child(
                            div()
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .right_0()
                                .bg(black().alpha(70.))
                                .text_color(white())
                                .text_sm()
                                .px_2()
                                .py_1()
                                .opacity(0.)
                                .group_hover(SharedString::new(file.id.as_str()), |el| {
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
            }))
    }
}
