use super::{AppError, models::user::res::UserResponse};

pub async fn get_user(token: &str) -> Result<UserResponse, AppError> {
    super::get("/user", token, None).await
}
