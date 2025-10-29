use gpui::*;
use gpui_component::{ActiveTheme, TitleBar};
use root::Rooter;

mod assets;
mod auth;
mod err;
mod root;
mod store;
mod theme;
mod util;

fn get_window_options(cx: &mut App) -> WindowOptions {
    let mut window_size = size(px(1600.0), px(1200.0));
    if let Some(display) = cx.primary_display() {
        let display_size = display.bounds().size;
        window_size.width = window_size.width.min(display_size.width * 0.85);
        window_size.height = window_size.height.min(display_size.height * 0.85);
    }
    let bounds = Bounds::centered(None, window_size, cx);
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitleBar::title_bar_options()),
        ..Default::default()
    }
}

fn main() {
    Application::new()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            let window_options = get_window_options(cx);

            let store = store::Store::load();
            cx.set_global(store);

            cx.activate(true);
            cx.open_window(window_options, |win, cx| {
                gpui_component::init(cx);
                gpui_component::theme::init(cx);
                theme::change_color_mode(cx.theme().mode, cx);

                let root_view = Rooter::view(win, cx);
                cx.new(|cx| gpui_component::Root::new(root_view.into(), win, cx))
            })
            .unwrap();
        });
}
