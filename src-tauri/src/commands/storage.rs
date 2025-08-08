use crate::commands::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn list_dir_items(token: &str, space_id: &str, path: &str) -> CommandResult {
    super::get(format!("/media/l/{path}"), token, Some(space_id)).await
}
