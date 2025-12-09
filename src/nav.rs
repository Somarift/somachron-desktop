use std::collections::HashMap;

use gpui::*;
use uuid::Uuid;

pub trait NavId: Render + 'static {
    fn id(&self) -> NavState;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NavState {
    pub space_id: Uuid,
    pub folder_id: Uuid,
}

impl NavState {
    pub fn new(space_id: Uuid, folder_id: Uuid) -> Self {
        Self {
            space_id,
            folder_id,
        }
    }

    pub fn with_folder(mut self, folder_id: Uuid) -> Self {
        self.folder_id = folder_id;
        self
    }
}

pub type Navigation = Entity<NavStack>;

pub enum NavEvent {
    Refresh,
    NewSpace(Uuid),
}
impl EventEmitter<NavEvent> for NavStack {}

pub struct NavStack {
    /// ptr - 1 = index
    ptr: usize,

    /// unique views
    views: HashMap<NavState, AnyView>,

    /// nav keys history
    stack: Vec<NavState>,
}

impl NavStack {
    pub fn new() -> Self {
        Self {
            ptr: 0,
            views: HashMap::new(),
            stack: Vec::with_capacity(512),
        }
    }

    pub fn current(&self) -> Option<&AnyView> {
        self.__current().and_then(|s| self.views.get(s))
    }

    pub fn current_space_id(&self) -> Option<&Uuid> {
        self.__current().map(|s| &s.space_id)
    }

    fn __current(&self) -> Option<&NavState> {
        if self.ptr == 0 {
            return None;
        }
        self.stack.iter().nth(self.ptr - 1)
    }

    pub fn at_begining(&self) -> bool {
        self.ptr == 0
    }

    pub fn at_end(&self) -> bool {
        self.ptr >= self.stack.len()
    }

    pub fn push<N: NavId, T: 'static>(&mut self, view: Entity<N>, cx: &mut Context<T>) {
        let view_id = view.read(cx).id();

        self.drop_later_and_views();

        self.stack.push(view_id.clone());
        if self.views.get(&view_id).is_none() {
            self.views.insert(view_id.clone(), view.into());
        }

        self.forward(cx);
        cx.notify();
    }

    pub fn forward<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.ptr = (self.ptr + 1).min(self.stack.len());
        cx.notify();
    }

    pub fn back<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.ptr = self.ptr.checked_sub(1).unwrap_or(0);
        cx.notify();
    }

    pub fn home<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.ptr = 0;
        cx.notify();
    }

    fn drop_later_and_views(&mut self) {
        // trim stack
        self.stack = self.stack.drain(..self.ptr).collect();

        // drop views who's state not in stack
        let mut views = HashMap::new();
        for state in self.stack.iter() {
            if let Some(_) = views.get(state) {
                // we already have it
                continue;
            }

            // else save if it has view
            if let Some(view) = self.views.get(state) {
                views.insert(state.clone(), view.clone());
            }
        }

        self.views = views;
    }
}
