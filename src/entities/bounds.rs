use gpui::*;

#[derive(Debug)]
pub struct SizeEvent;

#[derive(Debug)]
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
