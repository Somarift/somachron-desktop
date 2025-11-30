use crate::web::api::models::cloud::res::StreamedUrlsResponse;

use super::{
    AppError,
    models::cloud::res::{FileMetaReponse, FolderResponse},
};

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
) -> Result<Vec<FileMetaReponse>, AppError> {
    super::get(format!("/media/ls/{folder_id}"), token, Some(space_id)).await
}

pub async fn get_stream_urls(
    token: &str,
    space_id: &str,
    file_id: &str,
) -> Result<StreamedUrlsResponse, AppError> {
    super::get(format!("/media/stream/{file_id}"), token, Some(space_id)).await
}
