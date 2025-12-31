use uuid::Uuid;

use crate::web::api::{
    EmptyResponse,
    models::cloud::{
        req::{CreateFolderRequest, InitiateUploadRequest, UploadCompleteRequest},
        res::{InitiateUploadResponse, StreamedUrlResponse},
    },
};

use super::{
    AppError,
    models::cloud::res::{FileMetaReponse, FolderResponse},
};

pub async fn list_folders(token: &str, space_id: &Uuid, folder_id: &Uuid) -> Result<Vec<FolderResponse>, AppError> {
    super::get(format!("/media/lf/{folder_id}"), token, Some(space_id)).await
}

pub async fn list_files(token: &str, space_id: &Uuid, folder_id: &Uuid) -> Result<Vec<FileMetaReponse>, AppError> {
    super::get(format!("/media/ls/{folder_id}"), token, Some(space_id)).await
}

pub async fn list_gallery_files(token: &str, space_id: &Uuid) -> Result<Vec<FileMetaReponse>, AppError> {
    super::get("/media/lg", token, Some(space_id)).await
}

pub async fn get_folder(token: &str, space_id: &Uuid, folder_id: &Uuid) -> Result<FolderResponse, AppError> {
    super::get(format!("/media/d/{folder_id}"), token, Some(space_id)).await
}

pub async fn get_thumbnail_stream_url(
    token: &str,
    space_id: &Uuid,
    file_id: &Uuid,
) -> Result<StreamedUrlResponse, AppError> {
    super::get(format!("/media/stream/th/{file_id}"), token, Some(space_id)).await
}

pub async fn get_preview_stream_url(
    token: &str,
    space_id: &Uuid,
    file_id: &Uuid,
) -> Result<StreamedUrlResponse, AppError> {
    super::get(format!("/media/stream/{file_id}"), token, Some(space_id)).await
}

pub async fn get_download_stream_url(
    token: &str,
    space_id: &Uuid,
    file_id: &Uuid,
) -> Result<StreamedUrlResponse, AppError> {
    super::get(format!("/media/download/{file_id}"), token, Some(space_id)).await
}

pub async fn create_folder(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
    name: String,
) -> Result<EmptyResponse, AppError> {
    super::post(
        "/media/mkdir",
        token,
        Some(space_id),
        CreateFolderRequest {
            parent_folder_id: *folder_id,
            folder_name: name,
        },
    )
    .await
}

pub async fn delete_folder(token: &str, space_id: &Uuid, folder_id: &Uuid) -> Result<EmptyResponse, AppError> {
    super::delete(format!("/media/rm/{folder_id}"), token, Some(space_id)).await
}

pub async fn delete_file(token: &str, space_id: &Uuid, file_id: &Uuid) -> Result<EmptyResponse, AppError> {
    super::delete(format!("/media/rmf/{file_id}"), token, Some(space_id)).await
}

pub async fn init_file_upload(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
    name: &str,
) -> Result<InitiateUploadResponse, AppError> {
    super::post(
        "/media/upload",
        token,
        Some(space_id),
        InitiateUploadRequest {
            folder_id: *folder_id,
            file_name: name.to_owned(),
        },
    )
    .await
}

pub async fn complete_file_upload(
    token: &str,
    space_id: &Uuid,
    folder_id: &Uuid,
    name: &str,
    size: u64,
    millis: u64,
) -> Result<EmptyResponse, AppError> {
    super::post(
        "/media/upload/complete",
        token,
        Some(space_id),
        UploadCompleteRequest {
            folder_id: *folder_id,
            file_name: name.to_owned(),
            file_size: size,
            updated_millis: millis,
        },
    )
    .await
}
