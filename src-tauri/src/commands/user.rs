use crate::commands::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_profile(token: &str) -> CommandResult {
    super::get("/user", token, None).await
}
