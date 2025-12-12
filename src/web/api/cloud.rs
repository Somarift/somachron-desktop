use uuid::Uuid;

use crate::web::api::{
    EmptyResponse,
    models::cloud::{req::CreateFolderRequest, res::StreamedUrlsResponse},
};

use super::{
    AppError,
    models::cloud::res::{FileMetaReponse, FolderResponse},
};

pub async fn list_folders(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
) -> Result<Vec<FolderResponse>, AppError> {
    super::get(format!("/media/lf/{folder_id}"), token, Some(space_id)).await
}

pub async fn list_files(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
) -> Result<Vec<FileMetaReponse>, AppError> {
    super::get(format!("/media/ls/{folder_id}"), token, Some(space_id)).await
}

pub async fn get_folder(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
) -> Result<FolderResponse, AppError> {
    super::get(format!("/media/d/{folder_id}"), token, Some(space_id)).await
}

pub async fn get_stream_urls(
    token: &str,
    space_id: &Uuid,
    file_id: &Uuid,
) -> Result<StreamedUrlsResponse, AppError> {
    super::get(format!("/media/stream/{file_id}"), token, Some(space_id)).await
}

pub async fn create_folder(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
    name: String,
) -> Result<EmptyResponse, AppError> {
    super::post(
        "/media/mkdir",
        &token,
        Some(space_id),
        CreateFolderRequest {
            parent_folder_id: *folder_id,
            folder_name: name,
        },
    )
    .await
}

pub async fn delete_folder(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
) -> Result<EmptyResponse, AppError> {
    super::delete(format!("/media/rm/{folder_id}"), &token, Some(space_id)).await
}
