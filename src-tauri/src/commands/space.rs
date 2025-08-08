use serde::Serialize;

use super::CommandResult;

#[derive(Serialize)]
struct CreateSpace {
    name: String,
    description: String,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_user_spaces(token: &str) -> CommandResult {
    super::get("/space", token, None).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_space(token: &str, name: &str, description: &str) -> CommandResult {
    super::post(
        "/space",
        token,
        None,
        CreateSpace {
            name: name.to_owned(),
            description: description.to_owned(),
        },
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_space_users(token: &str, space_id: &str) -> CommandResult {
    super::get("/space/users", token, Some(space_id)).await
}
