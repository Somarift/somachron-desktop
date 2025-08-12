use serde::Serialize;
use tauri::State;

use crate::clerk::ClerkState;

use super::CommandResult;

#[derive(Serialize)]
struct CreateSpace {
    name: String,
    description: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_spaces(clerk: State<'_, ClerkState>) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get("/space", &token, None).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_space(
    clerk: State<'_, ClerkState>,
    name: &str,
    description: &str,
) -> CommandResult {
    let token = clerk.get_token().await?;
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
pub async fn get_space_users(clerk: State<'_, ClerkState>, space_id: &str) -> CommandResult {
    let token = clerk.get_token().await?;
    super::get("/space/users", &token, Some(space_id)).await
}
