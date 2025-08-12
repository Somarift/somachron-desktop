use serde::Serialize;
use tauri::State;

use crate::{clerk::ClerkState, commands::CommandResult};

#[derive(Serialize)]
struct CreateFolder {
    folder_path: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_dir_items(
    clerk: State<'_, ClerkState>,
    space_id: &str,
    path: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get(format!("/media/l/{path}"), &token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_folder(
    clerk: State<'_, ClerkState>,
    space_id: &str,
    path: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
    super::post(
        "/media/d",
        &token,
        Some(space_id),
        CreateFolder {
            folder_path: path.to_owned(),
        },
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_path(
    clerk: State<'_, ClerkState>,
    space_id: &str,
    path: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
    super::delete(format!("/media/p/{path}"), &token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_thumbnail(
    clerk: State<'_, ClerkState>,
    space_id: &str,
    file_id: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get(format!("/media/f/{file_id}"), &token, Some(space_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stream_signed_url(
    clerk: State<'_, ClerkState>,
    space_id: &str,
    file_id: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get(format!("/media/stream/{file_id}"), &token, Some(space_id)).await
}
