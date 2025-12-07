use std::sync::Arc;

use gpui::*;

pub struct NavContext<T> {
    state: WeakEntity<T>,
}

impl<T> Clone for NavContext<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<T: 'static> NavContext<T> {
    pub fn new(state: WeakEntity<T>) -> Self {
        Self { state }
    }

    pub fn update<R>(
        &self,
        cx: &mut Context<impl Render>,
        f: impl FnOnce(&mut T, &mut Context<T>) -> R,
    ) -> Option<R> {
        self.state.update(cx, f).ok()
    }
}

pub trait Navigation: Render + 'static {
    fn id(&self) -> impl Into<String>;
}

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
    stack: Vec<AnyView>,
    history: Vec<String>,
}

impl NavStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(512),
            history: Vec::with_capacity(512),
        }
    }

    pub fn current(&self) -> Option<&AnyView> {
        self.stack.last()
    }

    pub fn push<N: Navigation, T: 'static>(&mut self, view: Entity<N>, cx: &mut Context<T>) {
        let view_id = view.read(cx).id().into();
        self.stack.push(view.into());
        self.history.push(view_id);
        cx.notify();
    }

    pub fn pop<T: 'static>(&mut self, cx: &mut Context<T>) -> bool {
        if self.stack.is_empty() {
            return false;
        }
        self.stack.pop();
        self.history.pop();
        cx.notify();
        true
    }

    pub fn clear_and_push<N: Navigation, T: 'static>(
        &mut self,
        view: Entity<N>,
        cx: &mut Context<T>,
    ) {
        self.stack.clear();
        self.history.clear();
        self.push(view, cx);
    }
}
