use crate::{api::models::space::res::UserSpaceResponse, err::AppError};

pub async fn get_user_spaces(token: String) -> Result<Vec<UserSpaceResponse>, AppError> {
    super::get("/space", &token, None).await
}
