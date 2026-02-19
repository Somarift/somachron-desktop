use std::sync::Arc;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, WindowExt,
    button::{Button, ButtonVariants},
    notification::Notification,
};

use crate::{
    auth::Auth,
    entities::{
        media::{FetchMedia, MediaAssetState, MediaCacher, MediaState},
        nav::{NavId, NavState},
    },
    rt,
    ui::_components::loading_icon,
    util::MapAsync,
    web::api::{
        self,
        models::cloud::{MediaType, res::FileMetaReponse},
    },
};

actions!(media, [Left, Right]);
const MEDIA_CONTEXT: &str = "Media";

fn init_kb(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("left", Left, Some(MEDIA_CONTEXT))]);
    cx.bind_keys([KeyBinding::new("right", Right, Some(MEDIA_CONTEXT))]);
}

pub struct MediaUi {
    auth: Entity<Auth>,
    focus_handle: FocusHandle,
    current_nav: NavState,
    media_data: Entity<MediaState>,
    media_cacher: Entity<MediaCacher>,
    ptr: usize,
    loading: bool,
}

impl MediaUi {
    fn new(
        auth: Entity<Auth>,
        focus_handle: FocusHandle,
        current_nav: NavState,
        media_data: Entity<MediaState>,
        ptr: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let media_cacher = cx.new(|cx| MediaCacher::new(4, cx));
        Self {
            auth,
            focus_handle,
            current_nav,
            media_data,
            media_cacher,
            ptr,
            loading: false,
        }
    }

    pub fn view(
        auth: Entity<Auth>,
        current_nav: NavState,
        media_data: Entity<MediaState>,
        ptr: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            init_kb(cx);

            let fh = cx.focus_handle();
            fh.focus(window);

            Self::new(auth, fh, current_nav.for_media(), media_data, ptr, window, cx)
        })
    }

    fn left(&mut self, cx: &mut Context<Self>) {
        self.loading = false;
        self.media_data.update(cx, |md, cx| {
            let index = self.ptr.checked_sub(1).unwrap_or(self.ptr);

            if let Some((index, file)) = md.get_prev_file(index) {
                self.ptr = index;
                cx.emit(FetchMedia { file });
            }
            cx.notify();
        });
        cx.notify();
    }

    fn right(&mut self, cx: &mut Context<Self>) {
        self.loading = false;
        self.media_data.update(cx, |md, cx| {
            if let Some((index, file)) = md.get_next_file(self.ptr + 1) {
                self.ptr = index;
                cx.emit(FetchMedia { file });
            }
            cx.notify();
        });
        cx.notify();
    }

    fn fetch_play_video(&mut self, file: Arc<FileMetaReponse>, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();
        let nav_state = self.current_nav.clone();
        let file_id = file.id;

        let task = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| {
                    api::cloud::get_download_stream_url(&token, nav_state.space_id(), &file_id).await
                })
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading = false;
                match result {
                    Ok(data) => {
                        cx.background_spawn(async move {
                            let _ = std::process::Command::new("open")
                                .args(["-a", "quicktime player", data.url.as_str()])
                                .spawn();
                        })
                        .detach();
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
}

impl NavId for MediaUi {
    fn id(&self) -> NavState {
        self.current_nav.clone()
    }
}

impl Render for MediaUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(MEDIA_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Left, _window, cx| {
                this.left(cx);
            }))
            .on_action(cx.listener(|this, _: &Right, _window, cx| {
                this.right(cx);
            }))
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .p_2()
                    .pb_10()
                    .flex()
                    .size_full()
                    .gap_2()
                    .child(
                        Button::new("prev")
                            .icon(IconName::ChevronLeft)
                            .h_full()
                            .ghost()
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.left(cx);
                            })),
                    )
                    .when_some(
                        self.media_data.read_with(cx, |md, _cx| {
                            md.get_file(self.ptr)
                                .and_then(|f| md.asset(&f.id).cloned().map(|asset| (f, asset)))
                        }),
                        |this, (file, asset_state)| match asset_state {
                            MediaAssetState::Queued | MediaAssetState::Idle => this
                                .child(
                                    div()
                                        .size_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(loading_icon(|icon| icon.size_8())),
                                )
                                .size_full(),
                            MediaAssetState::Loaded { preview_path, .. } => this
                                .child(
                                    img(ImageSource::Resource(Resource::Path(preview_path.into())))
                                        .image_cache(&self.media_cacher)
                                        .id(SharedString::new(format!("{}", file.id)))
                                        .size_full()
                                        .rounded_md()
                                        .overflow_hidden()
                                        .object_fit(ObjectFit::Contain)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .with_loading(|| loading_icon(|icon| icon.size_8()).into_any_element()),
                                )
                                .when(matches!(file.media_type, MediaType::Video), |this| {
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
                                                    .id(SharedString::new(file.id.to_string()))
                                                    .p_6()
                                                    .rounded_full()
                                                    .bg(black().opacity(0.5))
                                                    .text_color(white())
                                                    .child(Icon::empty().path("icons/play.svg").size_8())
                                                    .cursor_pointer()
                                                    .hover(|this| this.bg(black().opacity(0.8)))
                                                    .on_click(cx.listener(move |this, _ev, window, cx| {
                                                        this.fetch_play_video(file.clone(), window, cx);
                                                    })),
                                            ),
                                    )
                                }),
                            MediaAssetState::Error(err) => this
                                .child(
                                    div()
                                        .size_full()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            Icon::new(IconName::TriangleAlert)
                                                .size_8()
                                                .text_color(cx.theme().danger),
                                        )
                                        .child(div().text_center().child(err.message)),
                                )
                                .size_full(),
                        },
                    )
                    .child(
                        Button::new("next")
                            .icon(IconName::ChevronRight)
                            .h_full()
                            .ghost()
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.right(cx);
                            })),
                    ),
            )
            .when_some(self.media_data.read(cx).get_file(self.ptr), |this, file| {
                this.child(
                    deferred(
                        div()
                            .px_2()
                            .py_1p5()
                            .absolute()
                            .bottom_0()
                            .w_full()
                            .flex()
                            .flex_shrink_0()
                            .border_t_1()
                            .border_color(cx.theme().sidebar_border)
                            .bg(cx.theme().sidebar)
                            .gap_4()
                            .child(div().text_xs().child(file.file_name.clone())),
                    )
                    .with_priority(999),
                )
            })
    }
}
