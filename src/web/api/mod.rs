use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{err::AppError, web::tokio_rt};

pub mod auth;
pub mod models;
pub mod space;
pub mod user;

pub const SIGN_UP_URL: &str = "https://somachron.shank03.com/auth/sign-up";

const API_URL: &str = "https://api-somachron.shank03.com/v1";
pub const REQ_ID_HEADER: &str = "x-sc-id";
const SPACE_ID_HEADER: &str = "X-Space-ID";

#[derive(Deserialize, Serialize)]
pub struct EmptyResponse {
    pub status: u16,
    pub message: String,
}

async fn get<R: DeserializeOwned>(
    api: impl Into<String>,
    token: &str,
    space_id: Option<&str>,
) -> Result<R, AppError> {
    tokio_rt(async move {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{API_URL}{}", api.into()))
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
            .send()
            .await
            .map_err(AppError::err)?;

        handle_response(res).await
    })
}

async fn post<R: DeserializeOwned, B: Serialize>(
    api: impl Into<String>,
    token: &str,
    space_id: Option<&str>,
    body: B,
) -> Result<R, AppError> {
    tokio_rt(async move {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{API_URL}{}", api.into()))
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
            .json(&body)
            .send()
            .await
            .map_err(AppError::err)?;

        handle_response(res).await
    })
}

async fn delete<R: DeserializeOwned>(
    api: impl Into<String>,
    token: &str,
    space_id: Option<&str>,
) -> Result<R, AppError> {
    tokio_rt(async move {
        let client = reqwest::Client::new();
        let res = client
            .delete(format!("{API_URL}{}", api.into()))
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
            .send()
            .await
            .map_err(AppError::err)?;

        handle_response(res).await
    })
}

async fn handle_response<R: DeserializeOwned>(res: reqwest::Response) -> Result<R, AppError> {
    let content_length = res.content_length().unwrap_or(0);
    if content_length == 0 {
        return Err(AppError {
            status: res.status().as_u16(),
            message: res.status().as_str().to_owned(),
            req_id: String::default(),
        });
    }

    if res.status().is_success() {
        res.json().await.map_err(AppError::err)
    } else {
        Err(AppError::from_res(res).await?)
    }
}
