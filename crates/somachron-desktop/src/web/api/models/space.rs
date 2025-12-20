use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SpaceRole {
    Owner,
    Read,
    Upload,
    Modify,
}

pub mod res {
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Deserialize, Clone)]
    pub struct SpaceResponse {
        pub id: Uuid,

        pub name: String,
        pub description: String,
        pub picture_url: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct UserSpaceResponse {
        pub id: Uuid,

        pub role: super::SpaceRole,
        pub space: SpaceResponse,
        pub folder: Uuid,
    }
}

pub mod req {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct CreateSpaceRequest {
        pub name: String,
        pub description: String,
    }
}
