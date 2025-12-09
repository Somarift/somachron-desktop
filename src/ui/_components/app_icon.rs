use gpui::*;
use gpui_component::{ActiveTheme, Icon};

pub fn comp(cx: &mut App, f: impl FnOnce(Icon) -> Icon) -> Div {
    let icon = icon(cx);

    div()
        .flex()
        .justify_center()
        .items_center()
        .bg(cx.theme().primary)
        .rounded_md()
        .p_0p5()
        .child(f(icon))
}

pub fn icon<'a>(cx: &'a mut App) -> Icon {
    Icon::new(Icon::empty())
        .text_color(cx.theme().primary_foreground)
        .path("icons/cloud-moon.svg")
}

pub fn btn(cx: &mut App, f: impl FnOnce(Icon) -> Icon) -> impl IntoElement {
    div()
}
