use serde::Serialize;

use crate::commands::CommandResult;

#[derive(Serialize)]
struct CreateFolder {
    folder_path: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_dir_items(token: &str, space_id: &str, path: &str) -> CommandResult {
    super::get(format!("/media/l/{path}"), token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_folder(token: &str, space_id: &str, path: &str) -> CommandResult {
    super::post(
        "/media/d",
        token,
        Some(space_id),
        CreateFolder {
            folder_path: path.to_owned(),
        },
    )
    .await
}
