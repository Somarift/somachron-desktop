use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
}

pub mod res {
    use chrono::{DateTime, Utc};
    use gpui::SharedString;
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Deserialize, Clone)]
    pub struct FolderResponse {
        pub id: Uuid,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,

        pub name: SharedString,
        pub path: SharedString,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct FileMetaReponse {
        pub id: Uuid,
        pub updated_at: DateTime<Utc>,

        pub file_name: SharedString,
        pub media_type: super::MediaType,
        // pub user: Option<String>,
        pub width: i32,
        // pub height: i32,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct StreamedUrlResponse {
        pub url: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct InitiateUploadResponse {
        pub url: SharedString,
        pub file_name: SharedString,
    }
}

pub mod req {
    use serde::Serialize;
    use uuid::Uuid;

    #[derive(Debug, Serialize)]
    pub struct CreateFolderRequest {
        pub parent_folder_id: Uuid,
        pub folder_name: String,
    }

    #[derive(Debug, Serialize)]
    pub struct InitiateUploadRequest {
        pub folder_id: Uuid,
        pub file_name: String,
    }

    #[derive(Debug, Serialize)]
    pub struct UploadCompleteRequest {
        pub folder_id: Uuid,
        pub file_name: String,
        pub file_size: u64,
        pub updated_millis: u64,
    }
}
