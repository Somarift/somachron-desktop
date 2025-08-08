use reqwest::Response;
use serde::{Deserialize, Serialize};

pub mod space;
pub mod storage;
pub mod user;

pub type CommandResult = Result<tauri::ipc::Response, IpcResult>;

pub const API_URL: &str = "http://localhost:8080/v1";
pub const REQ_ID_HEADER: &str = "x-sc-id";
pub const SPACE_ID_HEADER: &str = "X-Space-ID";

#[derive(Deserialize, Serialize)]
pub struct EmptyResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Serialize)]
pub struct IpcResult {
    pub status: u16,
    pub message: String,
    pub req_id: String,
}

impl IpcResult {
    pub async fn from_res(res: Response) -> Result<Self, Self> {
        let req_id = res
            .headers()
            .get(REQ_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_default();

        let data = res.json::<EmptyResponse>().await.map_err(Self::err)?;

        Ok(Self {
            status: data.status,
            message: data.message,
            req_id,
        })
    }

    pub fn err(err: impl std::error::Error) -> Self {
        log::error!("Error: {}", err);

        Self {
            status: 500,
            message: format!("{}", err),
            req_id: "".into(),
        }
    }
}

async fn get(api: impl Into<String>, token: &str, space_id: Option<&str>) -> CommandResult {
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.unwrap_or(""))
        .send()
        .await
        .map_err(IpcResult::err)?;

    let content_length = res.content_length().unwrap_or(0);
    if content_length == 0 {
        return Err(IpcResult {
            status: res.status().as_u16(),
            message: res.status().as_str().to_owned(),
            req_id: String::default(),
        });
    }

    if res.status().is_success() {
        res.bytes()
            .await
            .map(|bytes| tauri::ipc::Response::new(bytes.to_vec()))
            .map_err(IpcResult::err)
    } else {
        Err(IpcResult::from_res(res).await?)
    }
}
