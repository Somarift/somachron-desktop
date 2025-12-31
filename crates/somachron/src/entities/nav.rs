use std::collections::HashMap;

use gpui::*;
use uuid::Uuid;

pub trait NavId: Render + 'static {
    fn id(&self) -> NavState;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewType {
    Browse,
    Media,
    Gallery,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NavState {
    space_id: Uuid,
    folder_id: Uuid,
    ty: ViewType,
}

impl NavState {
    pub fn new(space_id: Uuid, folder_id: Uuid) -> Self {
        Self {
            space_id,
            folder_id,
            ty: ViewType::Browse,
        }
    }

    pub fn space_id(&self) -> &Uuid {
        &self.space_id
    }

    pub fn folder_id(&self) -> &Uuid {
        &self.folder_id
    }

    pub fn with_folder(mut self, folder_id: Uuid) -> Self {
        self.folder_id = folder_id;
        self
    }

    pub fn for_media(mut self) -> Self {
        self.ty = ViewType::Media;
        self
    }

    pub fn for_browse(mut self) -> Self {
        self.ty = ViewType::Browse;
        self
    }

    pub fn for_gallery(mut self) -> Self {
        self.ty = ViewType::Gallery;
        self
    }
}

pub type Navigation = Entity<NavStack>;

pub enum NavEvent {
    Refresh,
    RefreshView(NavState),
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
        self.stack.get(self.ptr - 1)
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
        if !self.views.contains_key(&view_id) {
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
        self.ptr = self.ptr.saturating_sub(1);
        cx.notify();
    }

    pub fn home<T: 'static>(&mut self, cx: &mut Context<T>) {
        self.ptr = 0;
        cx.notify();
    }

    /// Upon folder deletion, remove all the views related to the folder_id
    pub fn remove_folder_views<T: 'static>(&mut self, folder_id: Uuid, cx: &mut Context<T>) {
        // although not possible
        // if no current, just trim stack and views
        let Some(_) = self.__current().cloned() else {
            self.drop_later_and_views();
            cx.notify();
            return;
        };

        let mut entries = Vec::with_capacity(4);
        let mut stack = Vec::with_capacity(self.stack.len());

        for (i, state) in self.stack.iter().enumerate() {
            if state.folder_id == folder_id {
                // if there is view that is before current ptr, decrement it
                if i + 1 < self.ptr {
                    self.ptr = self.ptr.saturating_sub(1);
                }
                entries.push(state.clone());
            } else {
                stack.push(state.clone());
            }
        }

        // i know it's size will be 1 but who knows !
        // remove all views where folder_id matched
        for state in entries.into_iter() {
            let view = self.views.remove(&state);
            drop(view);
        }
        self.stack = stack;

        // yeah, i want to crash
        assert!(self.ptr <= self.stack.len());
        cx.notify();
    }

    fn drop_later_and_views(&mut self) {
        // trim stack
        self.stack = self.stack.drain(..self.ptr).collect();

        // drop views who's state not in stack
        let mut views = HashMap::new();
        for state in self.stack.iter() {
            if views.contains_key(state) {
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
