use commands::{auth, space, storage, user};
use tauri::Manager;

mod clerk;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(clerk::Clerk::init());
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            auth::setup_client,
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
