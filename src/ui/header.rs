use gpui::*;
use gpui_component::{
    ActiveTheme as _, Icon, Sizable, StyledExt, ThemeMode,
    button::{Button, ButtonVariants as _},
    label::Label,
};

use crate::theme::*;

const TITLE_BAR_LEFT_PADDING: Pixels = px(80.);

pub struct HeaderUi;

impl HeaderUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::new())
    }

    pub fn change_mode(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if cx.theme().mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        change_color_mode(new_mode, cx);
    }
}

impl Render for HeaderUi {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_toggle = Button::new("theme-mode")
            .icon(Icon::empty().path("icons/circle-shade.svg"))
            .small()
            .ghost()
            .on_click(cx.listener(Self::change_mode));

        div()
            .id("header-bar")
            .bg(cx.theme().title_bar)
            .pl(TITLE_BAR_LEFT_PADDING)
            .border_1()
            .border_b_1()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .bg(cx.theme().primary)
                                    .rounded_md()
                                    .size_5()
                                    .child(
                                        Icon::new(Icon::empty())
                                            .text_color(cx.theme().primary_foreground)
                                            .size_4()
                                            .path("icons/cloud-moon.svg"),
                                    ),
                            )
                            .child(Label::new("Somachron").text_xs().font_medium()),
                    )
                    .child(
                        div()
                            .pr(px(5.0))
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(theme_toggle),
                    ),
            )
    }
}
