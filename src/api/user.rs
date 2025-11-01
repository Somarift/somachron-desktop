use crate::{api::models::user::res::UserResponse, err::AppError};

pub async fn get_user(token: &str) -> Result<UserResponse, AppError> {
    super::get("/user", token, None).await
}
