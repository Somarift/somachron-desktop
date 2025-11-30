use serde::Serialize;
use tauri::State;

use crate::app::auth::AuthState;

use super::CommandResult;

#[derive(Serialize)]
struct CreateSpace {
    name: String,
    description: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_spaces(auth: State<'_, AuthState>) -> CommandResult {
    let token = auth.get_token().await?;
    super::get("/space", &token, None).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_space(
    auth: State<'_, AuthState>,
    name: &str,
    description: &str,
) -> CommandResult {
    let token = auth.get_token().await?;
    super::post(
        "/space",
        &token,
        None,
        CreateSpace {
            name: name.to_owned(),
            description: description.to_owned(),
        },
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_space_users(auth: State<'_, AuthState>, space_id: &str) -> CommandResult {
    let token = auth.get_token().await?;
    super::get("/space/users", &token, Some(space_id)).await
}
