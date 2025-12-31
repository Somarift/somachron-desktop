use crate::web::api::models::space::{req::CreateSpaceRequest, res::SpaceResponse};

use super::{AppError, models::space::res::UserSpaceResponse};

pub async fn get_user_spaces(token: String) -> Result<Vec<UserSpaceResponse>, AppError> {
    super::get("/space", &token, None).await
}

pub async fn create_space(token: String, name: String, description: String) -> Result<SpaceResponse, AppError> {
    super::post("/space", &token, None, CreateSpaceRequest { name, description }).await
}
