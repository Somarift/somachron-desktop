use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceRole {
    Owner,
    Read,
    Upload,
    Modify,
}

pub mod res {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct SpaceResponse {
        pub id: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,

        pub name: String,
        pub description: String,
        pub picture_url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct UserSpaceResponse {
        pub id: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,

        pub role: super::SpaceRole,
        pub space: SpaceResponse,
        pub folder: String,
    }
}
