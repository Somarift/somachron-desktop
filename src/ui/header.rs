use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    label::Label,
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt, ThemeMode,
};

use crate::theme::*;

const TITLE_BAR_LEFT_PADDING: Pixels = px(80.);

pub struct HeaderUi {}

impl HeaderUi {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }

    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
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

        let user_button = Button::new("user")
            .icon(IconName::CircleUser)
            .small()
            .ghost()
            .on_click(|_, _, cx| cx.open_url(""));

        div()
            .id("header-bar")
            .border_b_1()
            .bg(cx.theme().title_bar)
            .border_color(cx.theme().border)
            .pl(TITLE_BAR_LEFT_PADDING)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .p_1()
                    .child(Label::new("Somachron").text_xs().font_medium())
                    .child(
                        div()
                            .pr(px(5.0))
                            .flex()
                            .gap_2()
                            .items_center()
                            .child(theme_toggle)
                            .child(user_button),
                    ),
            )
    }
}
