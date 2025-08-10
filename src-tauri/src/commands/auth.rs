use tauri::State;

use crate::clerk::ClerkState;

use super::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn setup_client(clerk: State<'_, ClerkState>) -> CommandResult {
    clerk.setup_client().await?;
    Ok(tauri::ipc::Response::new(Vec::default()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sign_in(clerk: State<'_, ClerkState>, email: &str) -> CommandResult {
    let idn = clerk.sign_in(email).await?;
    clerk.prepare_first_factor(&idn).await?;

    Ok(tauri::ipc::Response::new(Vec::default()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn attempt_factor(clerk: State<'_, ClerkState>, code: &str) -> CommandResult {
    clerk.attempt_first_factor(code).await?;
    clerk.get_token().await?;

    Ok(tauri::ipc::Response::new(Vec::default()))
}
