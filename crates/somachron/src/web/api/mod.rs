use std::{path::PathBuf, sync::LazyLock};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::err::AppError;

pub mod auth;
pub mod cloud;
pub mod models;
pub mod space;
pub mod user;

pub const SIGN_UP_URL: &str = "https://somachron.shank03.com/auth/sign-up";

const API_URL: &str = "https://api-somachron.shank03.com/v1";
pub const REQ_ID_HEADER: &str = "x-sc-id";
const SPACE_ID_HEADER: &str = "X-Space-ID";

static API_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(super::make_http_client);

#[derive(Debug, Deserialize, Serialize)]
pub struct EmptyResponse {
    pub status: u16,
    pub message: String,
}

pub async fn download(url: String, dst: PathBuf) -> Result<PathBuf, AppError> {
    let res = API_CLIENT.get(url).send().await.map_err(|err| AppError::err(err))?;

    let status = res.status();

    if status.is_success() {
        let bytes = res.bytes().await.map_err(|err| AppError::err(err))?;
        let mut file = tokio::fs::File::create(&dst).await.map_err(|err| AppError::err(err))?;
        file.write_all(bytes.as_ref()).await.map_err(|err| AppError::err(err))?;

        tracing::info!(msg = "Downloaded file", path = format!("{}", dst.to_string_lossy()));

        return Ok(dst);
    }

    let text = res.text().await.map_err(|err| AppError::err(err))?;
    Err(AppError {
        status: status.as_u16(),
        message: text,
        req_id: "".into(),
    })
}

pub struct UploadRet {
    pub file_size: u64,
    pub updated_millis: u64,
}

pub async fn upload(url: &str, from: PathBuf) -> Result<UploadRet, AppError> {
    let file = tokio::fs::File::open(&from).await.map_err(|err| AppError::err(err))?;
    let metadata = file.metadata().await.map_err(|err| AppError::err(err))?;

    let file_size = metadata.len();
    let system_time = metadata.modified().map_err(|err| AppError::err(err))?;
    let updated_millis = system_time
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .map_err(|err| AppError::err(err))?;

    let stream = tokio_util::io::ReaderStream::with_capacity(file, 4096);
    let res = API_CLIENT
        .put(url)
        .header(
            reqwest::header::CONTENT_LENGTH,
            reqwest::header::HeaderValue::from_str(file_size.to_string().as_str()).unwrap(),
        )
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await
        .map_err(|err| AppError::err(err))?;

    let status = res.status();

    if status.is_success() {
        tracing::info!(msg = "Uploaded file", path = format!("{}", from.to_string_lossy()));
        return Ok(UploadRet {
            file_size,
            updated_millis,
        });
    }

    let text = res.text().await.map_err(|err| AppError::err(err))?;
    Err(AppError {
        status: status.as_u16(),
        message: text,
        req_id: "".into(),
    })
}

async fn get<R: DeserializeOwned + Send + 'static>(
    api: impl Into<String> + Send + 'static,
    token: &str,
    space_id: Option<&Uuid>,
) -> Result<R, AppError> {
    let res = API_CLIENT
        .get(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.map(|s| s.to_string()).unwrap_or_default())
        .send()
        .await
        .map_err(AppError::err)?;

    handle_response(res).await
}

async fn post<R: DeserializeOwned + Send + 'static, B: Serialize + Send + 'static>(
    api: impl Into<String> + Send + 'static,
    token: &str,
    space_id: Option<&Uuid>,
    body: B,
) -> Result<R, AppError> {
    let res = API_CLIENT
        .post(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.map(|s| s.to_string()).unwrap_or_default())
        .json(&body)
        .send()
        .await
        .map_err(AppError::err)?;

    handle_response(res).await
}

async fn delete<R: DeserializeOwned + Send + 'static>(
    api: impl Into<String> + Send + 'static,
    token: &str,
    space_id: Option<&Uuid>,
) -> Result<R, AppError> {
    let res = API_CLIENT
        .delete(format!("{API_URL}{}", api.into()))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header(SPACE_ID_HEADER, space_id.map(|s| s.to_string()).unwrap_or_default())
        .send()
        .await
        .map_err(AppError::err)?;

    handle_response(res).await
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
