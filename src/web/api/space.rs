use super::{AppError, models::space::res::UserSpaceResponse};

pub async fn get_user_spaces(token: String) -> Result<Vec<UserSpaceResponse>, AppError> {
    super::get("/space", &token, None).await
}
