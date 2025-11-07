use super::{AppError, models::cloud::res::FolderResponse};

pub async fn list_folders(
    token: &str,
    space_id: &str,
    folder_id: &str,
) -> Result<Vec<FolderResponse>, AppError> {
    super::get(format!("/media/lf/{folder_id}"), token, Some(space_id)).await
}

pub async fn list_files(
    token: &str,
    space_id: &str,
    folder_id: &str,
) -> Result<Vec<FolderResponse>, AppError> {
    super::get(format!("/media/ls/{folder_id}"), token, Some(space_id)).await
}
