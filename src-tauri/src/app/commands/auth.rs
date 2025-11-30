use tauri::State;

use crate::app::auth::AuthState;

use super::CommandResult;

#[tauri::command(rename_all = "snake_case")]
pub async fn setup_client(auth: State<'_, AuthState>) -> CommandResult {
    auth.setup_client().await?;
    Ok(tauri::ipc::Response::new(Vec::default()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn validate_auth(auth: State<'_, AuthState>) -> CommandResult {
    let state = match auth.fetch_token().await {
        Ok(_) => "app",
        Err(_) => "auth",
    };
    Ok(tauri::ipc::Response::new(state.as_bytes().to_vec()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sign_in(auth: State<'_, AuthState>, email: &str) -> CommandResult {
    let idn = auth.sign_in(email).await?;
    auth.prepare_first_factor(&idn).await?;

    Ok(tauri::ipc::Response::new(Vec::default()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn attempt_factor(auth: State<'_, AuthState>, code: &str) -> CommandResult {
    auth.attempt_first_factor(code).await?;
    let _ = auth.fetch_token().await?;

    Ok(tauri::ipc::Response::new(Vec::default()))
}
