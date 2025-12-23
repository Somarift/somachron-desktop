use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
};

use crate::{
    entities::{
        media::{FetchMedia, MediaAssetState, MediaState, PreviewAssetType},
        nav::{NavId, NavState},
    },
    ui::_components::loading_icon,
};

actions!(media, [Left, Right]);
const MEDIA_CONTEXT: &str = "Media";

fn init_kb(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("left", Left, Some(MEDIA_CONTEXT))]);
    cx.bind_keys([KeyBinding::new("right", Right, Some(MEDIA_CONTEXT))]);
}

pub struct MediaUi {
    focus_handle: FocusHandle,
    current_nav: NavState,
    media_data: Entity<MediaState>,
    ptr: usize,
}

impl MediaUi {
    fn new(
        focus_handle: FocusHandle,
        current_nav: NavState,
        media_data: Entity<MediaState>,
        ptr: usize,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus_handle,
            current_nav,
            media_data,
            ptr,
        }
    }

    pub fn view(
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

            Self::new(fh, current_nav.for_media(), media_data, ptr, window, cx)
        })
    }

    fn left(&mut self, cx: &mut Context<Self>) {
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
        self.media_data.update(cx, |md, cx| {
            if let Some((index, file)) = md.get_next_file(self.ptr + 1) {
                self.ptr = index;
                cx.emit(FetchMedia { file });
            }
            cx.notify();
        });
        cx.notify();
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
                        |this, (file, asset_state)| {
                            if let MediaAssetState::Loaded {
                                preview_asset_ty, ..
                            } = asset_state.clone()
                            {
                                match preview_asset_ty {
                                    PreviewAssetType::Loading => this
                                        .child(
                                            div()
                                                .size_full()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(loading_icon(|icon| icon.size_8())),
                                        )
                                        .size_full(),
                                    PreviewAssetType::Preview(path_buf) => {
                                        this.child(
                                            img(ImageSource::Resource(Resource::Path(
                                                path_buf.into(),
                                            )))
                                            .id(SharedString::new(format!("{}", file.id)))
                                            .size_full()
                                            .rounded_md()
                                            .overflow_hidden()
                                            // .image_cache(&self.image_cache)
                                            .object_fit(ObjectFit::Contain)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .with_loading(|| {
                                                loading_icon(|icon| icon.size_8())
                                                    .into_any_element()
                                            }),
                                        )
                                    }
                                    PreviewAssetType::VideoUrl(url) => {
                                        dbg!(&url.to_string());
                                        this.child(div().child("Video")).size_full()
                                    }
                                }
                            } else {
                                this
                            }
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
