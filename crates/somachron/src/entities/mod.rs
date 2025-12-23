use std::sync::Arc;

use crate::web::api::models::{space::res::UserSpaceResponse, user::res::UserResponse};

pub mod bounds;
pub mod media;
pub mod nav;
pub mod upload;

pub struct UserData {
    pub user_spaces: Vec<Arc<UserSpaceResponse>>,
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
