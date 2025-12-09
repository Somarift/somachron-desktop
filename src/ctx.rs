use gpui::*;

use crate::{
    nav::NavState,
    web::api::models::{space::res::UserSpaceResponse, user::res::UserResponse},
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

    // pub fn current_space(&self, nav_ctx) -> Option<&SpaceResponse> {
    //     self.nav_stack.current_space().and_then(|sp_id| {
    //         self.user_spaces.iter().find_map(|us| {
    //             if us.space.id == sp_id {
    //                 Some(&us.space)
    //             } else {
    //                 None
    //             }
    //         })
    //     })
    // }
}

pub type UserState = Entity<UserData>;

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
