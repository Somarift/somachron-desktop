use std::sync::Arc;

use gpui::*;
use gpui_component::{ActiveTheme, TitleBar};

mod assets;
mod auth;
mod err;
mod store;
mod theme;
mod ui;
mod util;
mod web;

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
        window_min_size: Some(size(px(800.0), px(600.0))),
        ..Default::default()
    }
}

fn main() {
    Application::new()
        .with_assets(assets::Assets)
        .with_http_client(Arc::new(web::WebClient::new()))
        .run(|cx: &mut App| {
            let window_options = get_window_options(cx);

            cx.bind_keys([KeyBinding::new("cmd-w", ui::CloseWindow, None)]);

            let store = store::Store::load();
            cx.set_global(store);

            cx.activate(true);
            cx.open_window(window_options, |win, cx| {
                gpui_component::init(cx);
                gpui_component::theme::init(cx);
                theme::change_color_mode(cx.theme().mode, cx);

                let root_view = ui::Rooter::view(win, cx);
                cx.new(|cx| gpui_component::Root::new(root_view.into(), win, cx))
            })
            .unwrap();
        });
}
