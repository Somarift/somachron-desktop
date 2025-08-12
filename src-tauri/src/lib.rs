use commands::{auth, space, storage, user};
use tauri::{menu::MenuItem, AppHandle, Manager};

mod clerk;
mod commands;

const CUSTOM_QUIT_MENU_ID: &str = "h_q";

fn replace_mac_quit<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    if let Some(menu) = app.menu() {
        if let Some(app_menu) = menu.items()?.first().and_then(|s| s.as_submenu()) {
            let last_index = app_menu.items()?.len() - 1;
            app_menu.remove_at(last_index)?;

            let quit_item = MenuItem::with_id(
                app.app_handle(),
                CUSTOM_QUIT_MENU_ID,
                "Quit",
                true,
                Some("Command+Q"),
            )?;
            app_menu.append(&quit_item)?;
        }
    }

    Ok(())
}

fn save_and_quit<R: tauri::Runtime>(app: &AppHandle<R>) {
    let clerk = app.state::<clerk::ClerkState>();
    tauri::async_runtime::block_on(async move {
        clerk.save_data().await;
    });

    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            app.manage(clerk::Clerk::init(app.handle().clone()));

            replace_mac_quit(app.app_handle())?;

            Ok(())
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            CUSTOM_QUIT_MENU_ID => {
                let Some(_) = app.get_webview_window("main") else {
                    return;
                };
                save_and_quit(app);
            }
            _ => (),
        })
        .on_window_event(|w, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                save_and_quit(w.app_handle());
            }
            _ => (),
        })
        .invoke_handler(tauri::generate_handler![
            auth::setup_client,
            auth::validate_auth,
            auth::sign_in,
            auth::attempt_factor,
            user::get_user_profile,
            space::get_user_spaces,
            space::get_space_users,
            space::create_space,
            storage::list_dir_items,
            storage::create_folder,
            storage::delete_path,
            storage::get_thumbnail,
            storage::get_stream_signed_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
