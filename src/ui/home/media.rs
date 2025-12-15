use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
};

use crate::{
    ctx::{FetchMedia, MediaData, UrlState},
    nav::{NavId, NavState},
    ui::_components::loading_icon,
    util,
};

pub struct MediaUi {
    current_nav: NavState,
    media_data: Entity<MediaData>,
    ptr: usize,
    image_cache: Entity<RetainAllImageCache>,
}

impl MediaUi {
    fn new(
        current_nav: NavState,
        media_data: Entity<MediaData>,
        ptr: usize,
        image_cache: Entity<RetainAllImageCache>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            current_nav: current_nav.for_media(),
            media_data,
            ptr,
            image_cache,
        }
    }

    pub fn view(
        current_nav: NavState,
        media_data: Entity<MediaData>,
        ptr: usize,
        image_cache: Entity<RetainAllImageCache>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(current_nav, media_data, ptr, image_cache, window, cx))
    }
}

impl NavId for MediaUi {
    fn id(&self) -> NavState {
        self.current_nav.clone()
    }
}

impl Render for MediaUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // div().flex().flex_col().size_full().child(
        div()
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
                                this.media_data.update(cx, |md, cx| {
                                    let index = this.ptr.checked_sub(1).unwrap_or(this.ptr);

                                    if let Some(id) = md.get(this.ptr).map(|(file, _)| file.id) {
                                        this.ptr = index;
                                        cx.emit(FetchMedia {
                                            file_id: id,
                                            index: this.ptr,
                                        });
                                    }
                                    cx.notify();
                                });
                            })),
                    )
                    .when_some(
                        self.media_data.update(cx, |md, _cx| md.get(self.ptr)),
                        |this, (file, url)| {
                            if let UrlState::Loaded(url) = url {
                                match file.media_type {
                                    crate::web::api::models::cloud::MediaType::Image => {
                                        this.child(
                                            img(ImageSource::Resource(Resource::Uri(
                                                SharedUri::from(&url.original_stream),
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
                                    crate::web::api::models::cloud::MediaType::Video => {
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
                                this.media_data.update(cx, |md, cx| {
                                    if let Some(id) = md.get(this.ptr + 1).map(|(file, _)| file.id)
                                    {
                                        this.ptr += 1;
                                        cx.emit(FetchMedia {
                                            file_id: id,
                                            index: this.ptr,
                                        });
                                    }
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .when_some(
                self.media_data.update(cx, |md, cx| md.get(self.ptr)),
                |this, (file, _)| {
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
                                .gap_4()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(file.file_name.clone()),
                                ),
                        )
                        .with_priority(999),
                    )
                },
            )
        // )
        // .when_some(
        //     self.media_data.update(cx, |md, cx| md.get(self.ptr)),
        //     |this, (file, url)| this.child(file.file_name.clone()),
        // )
    }
}
