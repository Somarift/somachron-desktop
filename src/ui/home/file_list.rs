use std::rc::Rc;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, ContextModal, IndexPath, label::Label, notification::Notification, v_flex,
    v_virtual_list,
};

use crate::{
    auth::Auth,
    ui::_components::{NavStack, NavState, loading_icon},
    util::MapAsync,
    web::api::{self, models::cloud::res::FileMetaReponse},
};

const PAGE_SIZE: usize = 20;

pub struct FileList {
    auth: Entity<Auth>,
    nav_top: Option<NavState>,

    file_list: Vec<FileMetaReponse>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    loading: bool,
    has_more: bool,

    _subscriptions: Vec<Subscription>,
}

impl FileList {
    pub fn new(
        auth: Entity<Auth>,
        nav_stack: Entity<NavStack>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let nav_sub = cx.subscribe_in(&nav_stack, window, |this, stack, event, window, cx| {
            let state = match event {
                crate::ui::_components::NavEvent::Reset(nav_state) => nav_state,
                crate::ui::_components::NavEvent::Push(nav_state) => nav_state,
            };

            this.nav_top = Some(state.clone());
            this.on_nav_fetch(window, cx);
            cx.notify();
        });

        Self {
            auth,
            nav_top: None,
            file_list: Vec::new(),
            item_sizes: Rc::new(Vec::new()),
            loading: false,
            has_more: false,
            _subscriptions: vec![nav_sub],
        }
    }

    pub fn view(
        auth: Entity<Auth>,
        nav_stack: Entity<NavStack>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, nav_stack, window, cx))
    }

    pub fn on_nav_fetch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.file_list.clear();
        self.loading = true;
        self.has_more = false;

        if let Some(state) = self.nav_top.as_ref().cloned() {
            cx.spawn_in(window, async move |this, cx| {
                let inner = this
                    .read_with(cx, |this, cx| this.auth.read(cx).inner())
                    .unwrap();

                let result = cx
                    .background_executor()
                    .spawn(async move {
                        inner
                            .get_token()
                            .await
                            .map_async(async move |token| {
                                api::cloud::list_files(&token, &state.space_id, &state.folder_id)
                                    .await
                            })
                            .await
                    })
                    .await;

                this.update_in(cx, |this, window, cx| {
                    match result {
                        Ok(files) => {
                            dbg!(files.len());
                            this.item_sizes = Rc::new(
                                (0..files.len()).map(|_| size(px(256.), px(30.))).collect(),
                            );
                            this.file_list = files;
                        }
                        Err(err) => window
                            .push_notification(Notification::error(err.message).autohide(true), cx),
                    };
                    cx.notify();
                })
                .unwrap();
            })
            .detach();
        }
    }

    // fn next_load(&mut self) {
    //     dbg!("next load", self.has_more);
    //     if !self.has_more {
    //         self.loading = false;
    //         return;
    //     }

    //     let start = self.list.len();
    //     let new = &self.server_list[start..((start + PAGE_SIZE).min(self.server_list.len()))];
    //     dbg!(new);
    //     self.list.extend_from_slice(new);

    //     self.loading = false;
    //     self.has_more = self.list.len() != self.server_list.len();
    // }
}

impl Render for FileList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().when(!self.file_list.is_empty(), |el| {
            let file_list = self.file_list.clone();

            el.child(uniform_list(
                "file_list",
                file_list.len(),
                move |view_range, _window, _cx| {
                    dbg!(&view_range);
                    view_range
                        .map(|i| {
                            dbg!(i);
                            let Some(item) = file_list.get(i) else {
                                return div().child("empty");
                            };

                            div().child(item.file_name.clone())
                        })
                        .collect()
                },
            ))

            // el.child(v_virtual_list(
            //     cx.entity(),
            //     "files_list",
            //     self.item_sizes.clone(),
            //     move |_this, visible_range, _window, _cx| {
            //         visible_range
            //             .map(|i| {
            //                 dbg!(i);
            //                 let Some(item) = files_list.get(i) else {
            //                     return div();
            //                 };
            //                 dbg!(&item);

            //                 div().child(item.file_name.clone())
            //             })
            //             .collect()
            //     },
            // ))
        })
    }
}

// impl ListDelegate for FileList {
//     type Item = ListItem;

//     fn items_count(&self, _section: usize, _cx: &App) -> usize {
//         self.server_list.len()
//     }

//     fn render_item(&self, ix: IndexPath, _window: &mut Window, cx: &mut App) -> Option<Self::Item> {
//         self.server_list.get(ix.row).map(|item| {
//             ListItem::new(ix)
//                 .child(Label::new(item.file_name.clone()))
//                 .bg(cx.theme().primary)
//                 .p_4()
//         })
//     }

//     fn set_selected_index(
//         &mut self,
//         ix: Option<IndexPath>,
//         _window: &mut Window,
//         cx: &mut Context<ListState<Self>>,
//     ) {
//         self.selected_index = ix;
//         cx.notify();
//     }

//     fn loading(&self, _cx: &App) -> bool {
//         self.loading
//     }

//     fn render_loading(&self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
//         v_flex()
//             .justify_center()
//             .items_center()
//             .w_full()
//             .p_4()
//             .child(loading_icon(|icon| icon.size_5()))
//     }

//     fn load_more(&mut self, window: &mut Window, cx: &mut Context<ListState<Self>>) {
//         if self.loading {
//             return;
//         }

//         println!("load more");
//         self.loading = true;
//         cx.spawn_in(window, async move |this, cx| {
//             let (is_empty, nav_top) = this
//                 .read_with(cx, |this, _cx| {
//                     let is_empty = this.delegate().server_list.is_empty();
//                     let nav_top = this.delegate().nav_top.clone();
//                     (is_empty, nav_top)
//                 })
//                 .unwrap();

//             if is_empty && let Some(state) = nav_top {
//                 Self::__fetch_files(&this, cx, state).await;
//             }

//             this.update(cx, |this, cx| {
//                 this.delegate_mut().next_load();
//                 cx.notify();
//             })
//             .unwrap();
//         })
//         .detach();
//     }
// }
