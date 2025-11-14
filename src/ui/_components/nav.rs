use std::sync::Arc;

use gpui::*;

#[derive(Debug, Clone)]
pub struct NavState {
    pub space_id: SharedString,
    pub folder_id: SharedString,
}

impl NavState {
    pub fn new(space_id: impl Into<Arc<str>>, folder_id: impl Into<Arc<str>>) -> Self {
        Self {
            space_id: SharedString::new(space_id),
            folder_id: SharedString::new(folder_id),
        }
    }
}

pub struct NavStack {
    stack: Vec<NavState>,
}

impl NavStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
        }
    }

    pub fn top(&self) -> Option<NavState> {
        self.stack.last().cloned()
    }

    pub fn on_event(&mut self, event: &NavEvent) {
        if let NavEvent::Reset(_) = event {
            self.stack.clear();
        }

        self.stack.push(
            match event {
                NavEvent::Reset(nav_state) => nav_state,
                NavEvent::Push(nav_state) => nav_state,
            }
            .clone(),
        );
    }
}

pub enum NavEvent {
    Reset(NavState),
    Push(NavState),
}

impl EventEmitter<NavEvent> for NavStack {}
