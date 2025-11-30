use serde::Serialize;
use tauri_plugin_http::reqwest;

use super::IpcError;

pub mod auth;
pub mod space;
pub mod user;

pub type CommandResult = Result<tauri::ipc::Response, IpcError>;

pub const API_URL: &str = "https://api-somachron.shank03.com/v1";
pub const REQ_ID_HEADER: &str = "x-sc-id";
pub const SPACE_ID_HEADER: &str = "X-Space-ID";

async fn get(api: impl Into<String>, token: &str, space_id: Option<&str>) -> CommandResult {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
        .send()
        .await
        .map_err(IpcError::err)?;

    handle_response(res).await
}

async fn post<B: Serialize>(
    api: impl Into<String>,
    token: &str,
    space_id: Option<&str>,
    body: B,
) -> CommandResult {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
        .json(&body)
        .send()
        .await
        .map_err(IpcError::err)?;

    handle_response(res).await
}

async fn delete(api: impl Into<String>, token: &str, space_id: Option<&str>) -> CommandResult {
    let client = reqwest::Client::new();
    let res = client
        .delete(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
        .send()
        .await
        .map_err(IpcError::err)?;

    handle_response(res).await
}

async fn handle_response(res: reqwest::Response) -> CommandResult {
    let content_length = res.content_length().unwrap_or(0);
    if content_length == 0 {
        return Err(IpcError {
            status: res.status().as_u16(),
            message: res.status().as_str().to_owned(),
            req_id: String::default(),
        });
    }

    if res.status().is_success() {
        res.bytes()
            .await
            .map(|bytes| tauri::ipc::Response::new(bytes.to_vec()))
            .map_err(IpcError::err)
    } else {
        Err(IpcError::from_res(res).await?)
    }
}
