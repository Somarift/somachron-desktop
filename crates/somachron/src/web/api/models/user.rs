pub mod res {
    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Deserialize, Clone)]
    pub struct UserResponse {
        pub id: Uuid,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub given_name: String,
        pub email: String,
        pub picture_url: String,
    }
}
