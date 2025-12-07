use tokio::task::JoinError;

#[derive(Debug)]
pub struct AppError {
    pub status: u16,
    pub message: String,
    pub req_id: String,
}

impl AppError {
    pub async fn from_res(res: reqwest::Response) -> Result<Self, Self> {
        let req_id = res
            .headers()
            .get(super::web::api::REQ_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_default();

        let data = res
            .json::<super::web::api::EmptyResponse>()
            .await
            .map_err(Self::err)?;

        Ok(Self {
            status: data.status,
            message: data.message,
            req_id,
        })
    }

    #[track_caller]
    pub fn message(message: impl Into<String>) -> Self {
        let message = message.into();

        let location = std::panic::Location::caller();
        // log::error!(
        //     "Error [{}:{}:{}]: {}",
        //     location.file(),
        //     location.line(),
        //     location.column(),
        //     message,
        // );

        Self {
            status: 400,
            message,
            req_id: "".into(),
        }
    }

    #[track_caller]
    pub fn err(err: impl std::error::Error) -> Self {
        let source = err.source();
        let location = std::panic::Location::caller();
        // log::error!(
        //     "Error [{}:{}:{}]: {}",
        //     location.file(),
        //     location.line(),
        //     location.column(),
        //     err
        // );

        Self {
            status: 500,
            message: format!("{} - {:?}", err, source),
            req_id: "".into(),
        }
    }

    #[track_caller]
    pub fn gp_err(err: anyhow::Error) -> Self {
        let location = std::panic::Location::caller();
        // log::error!(
        //     "Error [{}:{}:{}]: {}",
        //     location.file(),
        //     location.line(),
        //     location.column(),
        //     err
        // );

        Self {
            status: 500,
            message: format!("{}", err),
            req_id: "".into(),
        }
    }
}

impl From<JoinError> for AppError {
    fn from(value: JoinError) -> Self {
        AppError::err(value)
    }
}
