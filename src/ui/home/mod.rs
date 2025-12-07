use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Icon, IconName, Side, WindowExt,
    avatar::Avatar,
    h_flex,
    notification::Notification,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
};

use crate::{
    auth::Auth,
    rt,
    ui::{
        _components::{self, NavContext, NavStack, NavState, RenderBounds},
        home::browse::BrowseUi,
    },
    util::MapAsync,
    web::api::{
        self,
        models::{space::res::UserSpaceResponse, user::res::UserResponse},
    },
};

mod browse;

actions!(user, [MyAction, SignOut]);

pub struct HomeUi {
    auth: Entity<Auth>,
    nav_stack: Entity<NavStack>,
    nav_ctx: NavContext<NavStack>,
    render_bounds: Entity<RenderBounds>,

    user_spaces: Vec<UserSpaceResponse>,
    user: Option<UserResponse>,
    loading_sidebar: bool,
    loading_user: bool,
    _subscriptions: Vec<Subscription>,
}

impl HomeUi {
    fn new(auth: Entity<Auth>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let scroll_bounds = cx.new(|_cx| RenderBounds::new());
        let nav_stack = cx.new(|_cx| NavStack::new());
        let nav_ctx = NavContext::new(nav_stack.downgrade());

        let auth_sub = cx.subscribe_in(&auth, window, |this, _, event, window, cx| {
            match event {
                crate::auth::AuthEvent::Session(session_state) => match session_state {
                    crate::auth::SessionState::SignedIn => {
                        this.fetch_data(window, cx);
                    }
                    _ => (),
                },
                _ => (),
            };
        });

        Self {
            auth,
            nav_stack,
            nav_ctx,
            render_bounds: scroll_bounds,
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

    fn fetch_data(&self, window: &mut Window, cx: &mut Context<Self>) {
        let inner = self.auth.read(cx).inner();

        let _inner = inner.clone();
        let user_spaces = rt::spawn(cx, async move {
            _inner
                .get_token()
                .await
                .map_async(async move |token| api::space::get_user_spaces(token).await)
                .await
        });

        let user = rt::spawn(cx, async move {
            inner
                .get_token()
                .await
                .map_async(async move |token| api::user::get_user(&token).await)
                .await
        });

        cx.spawn_in(window, async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.loading_user = true;
                this.loading_sidebar = true;
                cx.notify();
            });

            let user_spaces = user_spaces.await.flatten();
            let user = user.await.flatten();

            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_sidebar = false;
                match user_spaces {
                    Ok(user_spaces) => {
                        this.user_spaces = user_spaces;
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(err.message).autohide(false),
                            cx,
                        );
                    }
                };

                this.loading_user = false;
                match user {
                    Ok(user) => {
                        this.user = Some(user);
                    }
                    Err(err) => {
                        window.push_notification(
                            Notification::error(&err.message)
                                .title("Failed to fetch user")
                                .autohide(false),
                            cx,
                        );
                    }
                };
                cx.notify();
            });
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
            .when_some(self.nav_stack.read(cx).current(), |el, view| {
                el.child(div().size_full().child(view.clone()))
            })
            .child({
                let this = cx.entity();
                canvas(
                    move |_, _, _| {},
                    move |el_bounds, _d, _w, cx| {
                        this.update(cx, |this, cx| {
                            this.render_bounds.update(cx, |bounds, cx| {
                                bounds.h_event(el_bounds.size.height).map(|ev| cx.emit(ev));
                            })
                        });
                    },
                )
            })
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
                                        .on_click(cx.listener(move |this, _ev, window, cx| {
                                            this.nav_ctx.update(cx, |ctx, cx| {
                                                ctx.clear_and_push(
                                                    BrowseUi::view(
                                                        this.auth.clone(),
                                                        this.nav_ctx.clone(),
                                                        this.render_bounds.clone(),
                                                        NavState::new(
                                                            us.space.id.clone(),
                                                            us.folder.clone(),
                                                        ),
                                                        window,
                                                        cx,
                                                    ),
                                                    cx,
                                                );
                                            });
                                        }))
                                }))
                            },
                        )
                    },
                )),
            )
    }
}
