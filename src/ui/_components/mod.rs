use gpui::*;
use gpui_component::{Icon, IconName};

pub mod nav;
pub use nav::*;

pub const MEDIA_HEIGHT: Pixels = px(176.);

pub fn loading_icon(f: impl FnOnce(Icon) -> Icon) -> impl IntoElement {
    f(Icon::new(IconName::LoaderCircle)).with_animation(
        ElementId::CodeLocation(*std::panic::Location::caller()),
        Animation::new(std::time::Duration::from_secs(2)).repeat(),
        |el, delta| el.transform(Transformation::rotate(percentage(delta))),
    )
}

#[derive(Debug)]
pub struct SizeEvent;

pub struct RenderBounds {
    pub height: Pixels,
    pub width: Pixels,
}

impl EventEmitter<SizeEvent> for RenderBounds {}

impl RenderBounds {
    pub fn new() -> Self {
        Self {
            height: px(0.),
            width: px(0.),
        }
    }

    pub fn h_event(&mut self, height: Pixels) -> Option<SizeEvent> {
        if self.height == height {
            return None;
        }
        self.height = height;
        Some(self.event())
    }

    pub fn w_event(&mut self, width: Pixels) -> Option<SizeEvent> {
        if self.width == width {
            return None;
        }
        self.width = width;
        Some(self.event())
    }

    fn event(&self) -> SizeEvent {
        SizeEvent
    }
}
