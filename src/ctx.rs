use std::sync::Arc;

use gpui::*;
use uuid::Uuid;

use crate::{
    nav::NavState,
    web::api::models::{
        cloud::res::{FileMetaReponse, StreamedUrlsResponse},
        space::res::UserSpaceResponse,
        user::res::UserResponse,
    },
};

pub struct UserData {
    pub user_spaces: Vec<UserSpaceResponse>,
    pub user: Option<UserResponse>,
}

impl UserData {
    pub fn new() -> Self {
        Self {
            user_spaces: Vec::new(),
            user: None,
        }
    }

    pub fn reset(&mut self) {
        self.user_spaces.clear();
        self.user = None;
    }
}

pub struct FetchMedia {
    pub file_id: Uuid,
    pub index: usize,
}

impl EventEmitter<FetchMedia> for MediaData {}

#[derive(Debug, Clone)]
pub enum UrlState {
    Idle,
    Queued,
    Loaded(StreamedUrlsResponse),
    Error,
}

pub struct MediaData {
    data: Vec<(Arc<FileMetaReponse>, UrlState)>,
}

impl MediaData {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn data(&self) -> &Vec<(Arc<FileMetaReponse>, UrlState)> {
        &self.data
    }

    pub fn data_mut(&mut self, index: usize) -> Option<&mut (Arc<FileMetaReponse>, UrlState)> {
        self.data.get_mut(index)
    }

    pub fn set_files(&mut self, files: Vec<FileMetaReponse>) {
        self.data = files
            .into_iter()
            .map(|f| (Arc::new(f), UrlState::Idle))
            .collect();
    }

    pub fn get(&mut self, index: usize) -> Option<(Arc<FileMetaReponse>, UrlState)> {
        self.data.get(index).cloned()
    }

    pub fn update_url(&mut self, index: usize, url: UrlState) {
        self.data.get_mut(index).map(|(_, u)| *u = url);
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }
}

// impl IntoIterator for MediaData {
//     type Item = (FileMetaReponse, UrlState);
//     type IntoIter = Zip<IntoIter<FileMetaReponse>, IntoIter<UrlState>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.files.into_iter().zip(self.urls)
//     }
// }

pub enum AppEvents {
    OpenDialog(NavState),
}

pub struct Contextor;

impl Global for Contextor {}

// pub struct Contextor<T> {
//     state: WeakEntity<T>,
// }

// impl<T> Clone for Contextor<T> {
//     fn clone(&self) -> Self {
//         Self {
//             state: self.state.clone(),
//         }
//     }
// }

// impl<T: 'static> Contextor<T> {
//     pub fn new(state: WeakEntity<T>) -> Self {
//         Self { state }
//     }

//     pub fn update<R>(
//         &self,
//         cx: &mut Context<impl Render>,
//         f: impl FnOnce(&mut T, &mut Context<T>) -> R,
//     ) -> Option<R> {
//         self.state.update(cx, f).ok()
//     }

//     pub fn read<R>(&self, cx: &impl AppContext, f: impl FnOnce(&T, &App) -> R) -> Option<R> {
//         self.state.read_with(cx, f).ok()
//     }
// }

// impl Contextor<AppState> {
//     pub fn read_auth(&self, cx: &App) {
//         let r = self.read(cx, |this, cx| this);
//     }
// }
