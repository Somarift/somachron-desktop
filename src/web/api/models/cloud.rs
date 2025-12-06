use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
}

pub mod res {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Clone)]
    pub struct FolderResponse {
        pub id: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,

        pub name: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct FileMetaReponse {
        pub id: String,

        pub file_name: String,
        pub media_type: super::MediaType,
        pub user: Option<String>,
        pub width: i32,
        pub height: i32,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct StreamedUrlsResponse {
        pub original_stream: String,
        pub thumbnail_stream: String,
    }
}
