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

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_path(token: &str, space_id: &str, path: &str) -> CommandResult {
    super::delete(format!("/media/p/{path}"), token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_thumbnail(token: &str, space_id: &str, file_id: &str) -> CommandResult {
    super::get(format!("/media/f/{file_id}"), token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stream_signed_url(token: &str, space_id: &str, file_id: &str) -> CommandResult {
    super::get(format!("/media/stream/{file_id}"), token, Some(space_id)).await
}
