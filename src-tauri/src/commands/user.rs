use tauri::State;

use crate::{clerk::ClerkState, commands::CommandResult};

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_profile(clerk: State<'_, ClerkState>) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get("/user", &token, None).await
}

pub async fn __get_user_profile(clerk: State<'_, ClerkState>) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get("/user", &token, None).await
}
