use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, ContextModal, Icon, IconName, StyledExt, h_virtual_list,
    notification::Notification, scroll::ScrollbarAxis,
};

use crate::{
    auth::Auth,
    ui::{
        _components::{self, NavEvent, NavStack, NavState},
        home::file_list::FileList,
    },
    util::MapAsync,
    web::api::{self, models::cloud::res::FolderResponse},
};

pub struct BrowseUi {
    auth: Entity<Auth>,
    nav_stack: Entity<NavStack>,
    file_list: Entity<FileList>,

    current_nav: Option<NavState>,
    folders: Vec<FolderResponse>,
    loading: bool,
    _subscriptions: Vec<Subscription>,
}

impl BrowseUi {
    fn new(
        auth: Entity<Auth>,
        nav_stack: Entity<NavStack>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let file_list = FileList::view(auth.clone(), nav_stack.clone(), window, cx);

        let nav_sub = cx.subscribe_in(&nav_stack, window, |this, _entity, event, window, cx| {
            this.nav_stack.update(cx, |stack, cx| {
                this.folders.clear();

                stack.on_event(event);
                this.current_nav = stack.top();
                cx.notify();
            });

            this.fetch_fs(window, cx);
        });

        Self {
            auth,
            nav_stack,
            file_list,
            folders: Vec::new(),
            current_nav: None,
            loading: false,
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

    fn fetch_fs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.current_nav.as_ref().cloned() else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .update(cx, |this, cx| {
                    this.loading = true;
                    cx.notify();

                    this.auth.read(cx).inner()
                })
                .unwrap();

            let folders = cx
                .background_executor()
                .spawn(async move {
                    inner
                        .get_token()
                        .await
                        .map_async(async move |token| {
                            api::cloud::list_folders(&token, &state.space_id, &state.folder_id)
                                .await
                        })
                        .await
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.loading = false;
                match folders {
                    Ok(folders) => this.folders = folders,
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

impl Render for BrowseUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("browse_ui")
            .scrollable(ScrollbarAxis::Vertical)
            .p_4()
            .when(self.loading, |el| {
                el.child(_components::loading_icon(|icon| icon.size_5()))
            })
            .when_some(self.current_nav.clone(), |el, state| {
                el.child(state.space_id)
                    .child(state.folder_id)
                    .when(!self.folders.is_empty(), |el| {
                        el.child(self.render_folder_cards(cx))
                    })
                    .child(self.file_list.clone())
            })
    }
}

/// UI functions
impl BrowseUi {
    fn render_folder_cards(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .flex_auto()
            .gap_2()
            .children(self.folders.iter().cloned().map(|folder| {
                div()
                    .flex_auto()
                    .id(SharedString::new(folder.id.clone()))
                    .on_click(cx.listener(move |this, _ev, _window, cx| {
                        this.nav_stack.update(cx, |_stack, cx| {
                            cx.emit(NavEvent::Push(NavState::new(
                                this.current_nav.as_ref().unwrap().space_id.clone(),
                                folder.id.clone(),
                            )));
                        });
                    }))
                    .border_1()
                    .rounded_lg()
                    .p_4()
                    .bg(cx.theme().accent)
                    .child(
                        div()
                            .flex()
                            .justify_start()
                            .items_center()
                            .gap_4()
                            .child(Icon::new(IconName::Folder).size_4())
                            .child(div().text_sm().flex_wrap().font_medium().child(folder.name)),
                    )
            }))
    }
}
