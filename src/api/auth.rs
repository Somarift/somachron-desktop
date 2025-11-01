use crate::err::AppError;

pub async fn sync_auth(token: &str) -> Result<(), AppError> {
    super::post("/auth/sync", token, None, ()).await
}
