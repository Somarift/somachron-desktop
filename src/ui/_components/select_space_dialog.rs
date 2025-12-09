use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, StyledExt, WindowExt, button::Button, dialog::Dialog,
    input::InputState, scroll::ScrollbarAxis, v_flex,
};

use crate::{
    nav::NavState, ui::_components::create_space_dialog,
    web::api::models::space::res::UserSpaceResponse,
};

pub fn comp<T: create_space_dialog::CreateSpaceDialog + 'static>(
    dialog: Dialog,
    entity: WeakEntity<T>,
    user_spaces: Vec<UserSpaceResponse>,
    cx: &mut App,
    on_space_select: impl Fn(WeakEntity<T>, NavState, &mut Window, &mut App) + Clone + 'static,
) -> Dialog {
    let _entity = entity.clone();

    dialog
        .rounded_lg()
        .title("Select space")
        .v_flex()
        .max_h_128()
        .footer(move |_, _, _, _| {
            let entity = _entity.clone();
            vec![create_space_dialog::trigger(
                entity,
                gpui_component::Size::default(),
            )]
        })
        .child(
            div()
                .scrollable(ScrollbarAxis::Vertical)
                .flex()
                .flex_col()
                .gap_2()
                .children(user_spaces.iter().map(move |us| {
                    let entity = entity.clone();
                    let nav_state = NavState::new(us.space.id.clone(), us.folder.clone());
                    let on_space_select = on_space_select.clone();

                    div()
                        .id(SharedString::new(us.space.id.to_string()))
                        .flex()
                        .items_center()
                        .bg(cx.theme().sidebar)
                        .rounded_md()
                        .p_2()
                        .gap_4()
                        .hover(|el| el.bg(cx.theme().secondary_hover))
                        .child(Icon::new(IconName::GalleryVerticalEnd))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .truncate()
                                .text_ellipsis()
                                .child(
                                    v_flex()
                                        .whitespace_normal()
                                        .text_sm()
                                        .child(us.space.name.clone())
                                        .font_medium(),
                                )
                                .child(
                                    v_flex()
                                        .whitespace_normal()
                                        .text_sm()
                                        .child(if us.space.description.is_empty() {
                                            String::from("No description")
                                        } else {
                                            us.space.description.clone()
                                        })
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .on_click(move |_ev, window, cx| {
                            on_space_select(entity.clone(), nav_state.clone(), window, cx)
                        })
                })),
        )
}
