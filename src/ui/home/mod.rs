use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, ContextModal, Icon, IconName, Side,
    avatar::Avatar,
    h_flex,
    notification::Notification,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};

use crate::{
    auth::Auth,
    ui::{
        _components::{self, NavStack, NavState},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api::{
        self,
        models::{space::res::UserSpaceResponse, user::res::UserResponse},
    },
};

mod browse;
mod file_list;

actions!(user, [MyAction, SignOut]);

pub struct HomeUi {
    auth: Entity<Auth>,
    nav_stack: Entity<NavStack>,

    browse_ui: Entity<BrowseUi>,

    user_spaces: Vec<UserSpaceResponse>,
    user: Option<UserResponse>,
    loading_sidebar: bool,
    loading_user: bool,
    _subscriptions: Vec<Subscription>,
}

impl HomeUi {
    fn new(auth: Entity<Auth>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let nav_stack = cx.new(|_cx| NavStack::new());

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                crate::auth::AuthEvent::Session(session_state) => match session_state {
                    crate::auth::SessionState::SignedIn => {
                        this.fetch_user(window, cx);
                        this.fetch_user_spaces(window, cx);
                    }
                    _ => (),
                },
                _ => (),
            };
        });

        let browse_ui = BrowseUi::view(auth.clone(), nav_stack.clone(), window, cx);

        Self {
            auth,
            nav_stack,
            browse_ui,
            user_spaces: Vec::new(),
            user: None,
            loading_sidebar: false,
            loading_user: false,
            _subscriptions: vec![auth_sub],
        }
    }

    pub fn view(auth: Entity<Auth>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(auth, window, cx))
    }

    fn fetch_user_spaces(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .update(cx, |this, cx| {
                    this.loading_sidebar = true;
                    cx.notify();

                    this.auth.read(cx).inner()
                })
                .unwrap();

            let user_spaces = cx
                .background_executor()
                .spawn(async move {
                    inner
                        .get_token()
                        .await
                        .map_async(async move |token| api::space::get_user_spaces(token).await)
                        .await
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.loading_sidebar = false;
                match user_spaces {
                    Ok(user_spaces) => {
                        this.user_spaces = user_spaces;
                    }
                    Err(err) => {
                        window
                            .push_notification(Notification::error(err.message).autohide(true), cx);
                    }
                };
                cx.notify();
            })
            .unwrap();
        })
        .detach();
    }

    fn fetch_user(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let inner = this
                .update(cx, |this, cx| {
                    this.loading_user = true;
                    cx.notify();

                    this.auth.read(cx).inner()
                })
                .unwrap();

            let user = cx
                .background_executor()
                .spawn(async move {
                    inner
                        .get_token()
                        .await
                        .map_async(async move |token| api::user::get_user(&token).await)
                        .await
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.loading_user = false;
                match user {
                    Ok(user) => {
                        this.user = Some(user);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(&err.message)
                                .title("Failed to fetch user")
                                .autohide(true),
                            cx,
                        );
                    }
                };
                cx.notify();
            })
            .unwrap();
        })
        .detach();
    }
}

impl Render for HomeUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .child(
                div()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_br_xl()
                    .rounded_tr_xl()
                    .child(self.render_sidebar(cx)),
            )
            .child(div().size_full().child(self.browse_ui.clone()))
    }
}

impl HomeUi {
    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        Sidebar::new(Side::Left)
            .border_width(0.)
            .header(SidebarHeader::new().when_else(
                self.loading_user,
                |el| {
                    el.child(
                        h_flex()
                            .gap_2()
                            .text_color(cx.theme().muted_foreground)
                            .child(_components::loading_icon(|icon| icon.size_6()))
                            .child("Loading"),
                    )
                },
                |el| {
                    el.when_none(&self.user, |el| {
                        el.child(h_flex().gap_2().child("No user :/"))
                    })
                    .when_some(self.user.clone(), |el, user| {
                        el.child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Avatar::new()
                                                .name(&user.given_name)
                                                .src(user.picture_url)
                                                .size_8(),
                                        )
                                        .child(user.given_name),
                                )
                                .child(Icon::new(IconName::ChevronsUpDown).size_4()),
                        )
                    })
                },
            ))
            .child(
                SidebarGroup::new("Spaces").child(SidebarMenu::new().when_else(
                    self.loading_sidebar,
                    |el| {
                        el.child(
                            SidebarMenuItem::new("Loading")
                                .active(false)
                                .suffix(_components::loading_icon(|icon| icon.size_4())),
                        )
                    },
                    |el| {
                        el.when_else(
                            self.user_spaces.is_empty(),
                            |el| el.child(SidebarMenuItem::new("No spaces")),
                            |el| {
                                el.children(self.user_spaces.iter().cloned().map(|us| {
                                    SidebarMenuItem::new(&us.space.name)
                                        .icon(Icon::new(IconName::GalleryVerticalEnd))
                                        .on_click(cx.listener(move |this, _ev, _window, cx| {
                                            this.nav_stack.update(cx, |_stack, cx| {
                                                cx.emit(_components::NavEvent::Reset(
                                                    NavState::new(
                                                        us.space.id.clone(),
                                                        us.folder.clone(),
                                                    ),
                                                ));
                                            })
                                        }))
                                }))
                            },
                        )
                    },
                )),
            )
    }
}
