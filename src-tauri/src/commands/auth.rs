use tauri::State;

use crate::clerk::ClerkState;

use super::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn setup_client(clerk: State<'_, ClerkState>) -> CommandResult {
    clerk.setup_client().await?;
    Ok(tauri::ipc::Response::new(Vec::default()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn validate_auth(clerk: State<'_, ClerkState>) -> CommandResult {
    let state = match clerk.fetch_token().await {
        Ok(_) => "app",
        Err(_) => "auth",
    };
    Ok(tauri::ipc::Response::new(state.as_bytes().to_vec()))
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
    let _ = clerk.fetch_token().await?;

    Ok(tauri::ipc::Response::new(Vec::default()))
}
