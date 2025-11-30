use tauri::State;

use crate::app::auth::AuthState;

use super::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_profile(auth: State<'_, AuthState>) -> CommandResult {
    let token = auth.get_token().await?;
    super::get("/user", &token, None).await
}

pub async fn __get_user_profile(auth: State<'_, AuthState>) -> CommandResult {
    let token = auth.get_token().await?;
    super::get("/user", &token, None).await
}
