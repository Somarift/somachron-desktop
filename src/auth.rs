use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use base64::Engine;
use gpui::*;
use reqwest::header::{HOST, ORIGIN, REFERER, USER_AGENT};
use serde::Deserialize;

use crate::{
    err::AppError,
    store::{Cookie, Store},
    util,
};

const CLERK_API_URL: &str = "https://clerk.somachron.shank03.com";
const CLERK_QUERY_VERSION: &str = "?__clerk_api_version=2025-04-10";
const CLERK_STRATEGY: &str = "email_code";

const SET_COOKIE_HEADER: &str = "Set-Cookie";
const COOKIE_HEADER: &str = "Cookie";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const FORM_DATA_TYPE: &str = "application/x-www-form-urlencoded";

#[derive(Debug)]
pub enum AuthClientEvent {
    Loading,
    Setup(Result<(), AppError>),
}

#[derive(Debug, Clone)]
pub enum SessionState {
    Validating,
    SignedIn,
    LoggedOut,
}

#[derive(Debug)]
pub enum AuthEvent {
    Client(AuthClientEvent),
    Session(SessionState),
}

pub struct Auth {
    _inner: Arc<InnerAuth>,
}

impl EventEmitter<AuthEvent> for Auth {}

impl Auth {
    pub fn init(cx: &mut App) -> Self {
        let inner = InnerAuth::init(cx);

        Self {
            _inner: Arc::new(inner),
        }
    }

    pub fn inner(&self) -> Arc<InnerAuth> {
        self._inner.clone()
    }

    pub fn save(&self, cx: &mut App) {
        self._inner.save_data(cx);
    }
}

#[derive(Clone)]
struct AuthToken {
    token: String,
    exp: u64, // secs
}

impl AuthToken {
    fn new(token: String) -> Result<Self, AppError> {
        let payload = match token.split('.').nth(1) {
            Some(p) => base64::prelude::BASE64_STANDARD_NO_PAD
                .decode(p)
                .map_err(AppError::err)?,
            None => return Err(AppError::message("Invalid token")),
        };

        let __JwtClaims { exp } = serde_json::from_slice(&payload).map_err(AppError::err)?;

        Ok(Self { token, exp })
    }
}

pub struct InnerAuth {
    client: reqwest::Client,
    rt: tokio::runtime::Handle,
    cookies: RwLock<HashMap<String, Cookie>>,

    client_id: RwLock<Option<String>>,
    sign_in_id: RwLock<Option<String>>,
    session_id: RwLock<Option<String>>,

    token_spec: RwLock<Option<AuthToken>>,
}

impl InnerAuth {
    fn init(cx: &mut App) -> Self {
        let client = reqwest::ClientBuilder::new()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.append(HOST, "clerk.somachron.shank03.com".parse().unwrap());
                headers.append(ORIGIN, "https://somachron.shank03.com".parse().unwrap());
                headers.append(REFERER, "https://somachron.shank03.com".parse().unwrap());

                headers.append(
                    USER_AGENT,
                    format!(
                        "Somachron-Desktop/version ({}-{})",
                        util::os_name(),
                        util::os_version()
                    )
                    .parse()
                    .unwrap(),
                );
                headers
            })
            .build()
            .unwrap();

        let rt = super::web::get_tokio_rt();

        let store = Store::global_get(cx);
        let cookies = store.cookies.clone();
        let cookies = filter_store_cookies(cookies);
        let client_id = store.client_id.clone();
        let session_id = store.session_id.clone();

        Self {
            client,
            rt,
            cookies: RwLock::new(cookies),
            client_id: RwLock::new(client_id),
            session_id: RwLock::new(session_id),
            sign_in_id: RwLock::new(None),
            token_spec: RwLock::new(None),
        }
    }

    pub fn has_session(&self) -> bool {
        self.session_id.read().unwrap().is_some()
    }

    pub async fn setup_client(&self) -> Result<(), AppError> {
        self.rt.block_on(async move {
            {
                let rl = self.client_id.read().unwrap();
                if let Some(_) = &*rl {
                    return Ok(());
                }
            }

            // get environment
            let res = self
                .client
                .get(format!(
                    "{CLERK_API_URL}/v1/environment{CLERK_QUERY_VERSION}"
                ))
                .send()
                .await
                .expect("Failed to send env request");

            {
                let cookies = extract_cookies(res.headers());
                let mut wl = self.cookies.write().unwrap();
                wl.extend(cookies);
            }

            if res.status().is_client_error() {
                let text = res.text().await.unwrap();
                return Err(AppError::message(format!("Error setting up env: {}", text)));
            }

            // get client
            let res = self
                .client
                .get(format!("{CLERK_API_URL}/v1/client{CLERK_QUERY_VERSION}"))
                .send()
                .await
                .expect("Failed to send client request");

            let cookies = extract_cookies(res.headers());
            if res.status().is_client_error() {
                let text = res.text().await.map_err(AppError::err)?;
                return Err(AppError::message(format!("Error setting up env: {}", text)));
            }

            let data: ClientPayload = res.json().await.unwrap();

            {
                let mut wl = self.client_id.write().unwrap();
                *wl = Some(data.response.id);
            }

            self.update_cookies(cookies, data.response.cookie_expires_at);

            Ok(())
        })
    }

    pub async fn sign_in(&self, email: &str) -> Result<String, AppError> {
        self.rt.block_on(async move {
            let res = self
                .client
                .post(format!(
                    "{CLERK_API_URL}/v1/client/sign_ins{CLERK_QUERY_VERSION}"
                ))
                .header(COOKIE_HEADER, self.get_header_cookies())
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[("identifier", email)])
                .send()
                .await
                .map_err(AppError::err)?;

            let cookies = extract_cookies(res.headers());

            if res.status().is_success() {
                let data: SignInPayload = res.json().await.map_err(AppError::err)?;
                let idn = data
                    .response
                    .supported_first_factors
                    .and_then(|factors| {
                        factors
                            .into_iter()
                            .find(|f| f.primary && f.strategy.as_str().cmp(CLERK_STRATEGY).is_eq())
                    })
                    .ok_or(AppError::message("No primary factor found"))?;

                {
                    let mut guard = self.sign_in_id.write().unwrap();
                    *guard = Some(data.response.id);
                }
                self.update_cookies(cookies, data.client.cookie_expires_at);

                Ok(idn.email_address_id)
            } else {
                let err = self.get_error(res).await?;
                Err(err)
            }
        })
    }

    pub async fn prepare_first_factor(&self, email_address_id: &str) -> Result<(), AppError> {
        self.rt.block_on(async move {
            let sia = {
                let g_sia = self.sign_in_id.read().unwrap();
                match &*g_sia {
                    Some(s) => s.clone(),
                    None => return Err(AppError::message("No sign in instance")),
                }
            };

            let res = self
                .client
                .post(format!(
                "{CLERK_API_URL}/v1/client/sign_ins/{sia}/prepare_first_factor{CLERK_QUERY_VERSION}"
            ))
                .header(COOKIE_HEADER, self.get_header_cookies())
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[
                    ("email_address_id", email_address_id),
                    ("strategy", CLERK_STRATEGY),
                ])
                .send()
                .await
                .map_err(AppError::err)?;

            let cookies = extract_cookies(res.headers());

            if res.status().is_success() {
                let data: SignInPayload = res.json().await.map_err(AppError::err)?;
                self.update_cookies(cookies, data.client.cookie_expires_at);
                Ok(())
            } else {
                let err = self.get_error(res).await?;
                Err(err)
            }
        })
    }

    pub async fn attempt_first_factor(&self, code: &str) -> Result<(), AppError> {
        self.rt.block_on(async move {
            let sia = {
                let g_sia = self.sign_in_id.read().unwrap();
                match &*g_sia {
                    Some(s) => s.clone(),
                    None => return Err(AppError::message("No sign in instance")),
                }
            };

            let res = self
                .client
                .post(format!(
                "{CLERK_API_URL}/v1/client/sign_ins/{sia}/attempt_first_factor{CLERK_QUERY_VERSION}"
            ))
                .header(COOKIE_HEADER, self.get_header_cookies())
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[("code", code), ("strategy", CLERK_STRATEGY)])
                .send()
                .await
                .map_err(AppError::err)?;

            let cookies = extract_cookies(res.headers());

            if res.status().is_success() {
                let data: SignInPayload = res.json().await.map_err(AppError::err)?;

                let session_id = data
                    .response
                    .created_session_id
                    .ok_or(AppError::message("No created session ID"))?;

                {
                    let mut wl = self.session_id.write().unwrap();
                    *wl = Some(session_id);
                }

                self.update_cookies(cookies, data.client.cookie_expires_at);
                Ok(())
            } else {
                let err = self.get_error(res).await?;
                Err(err)
            }
        })
    }

    pub async fn get_token(&self) -> Result<String, AppError> {
        let spec = {
            let gt = self.token_spec.read().unwrap();
            match &*gt {
                Some(spec) => spec.clone(),
                None => return Err(AppError::message("No token !")),
            }
        };

        let current_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .and_then(|v| Ok(v.as_secs()))
            .unwrap_or(1);

        if current_secs <= spec.exp {
            return Ok(spec.token);
        }

        println!(
            "Fetching token: current: {}, exp: {}",
            current_secs, spec.exp
        );
        self.fetch_token().await?;

        let spec = {
            let gt = self.token_spec.read().unwrap();
            match &*gt {
                Some(spec) => spec.clone(),
                None => return Err(AppError::message("No token !")),
            }
        };

        Ok(spec.token)
    }

    pub async fn fetch_token(&self) -> Result<(), AppError> {
        self.rt.block_on(async move {
            let sid = {
                let rl = self.session_id.read().unwrap();
                match &*rl {
                    Some(sid) => sid.clone(),
                    None => return Err(AppError::message("No session id")),
                }
            };

            let mut wl = self.token_spec.write().unwrap();

            let res = self
                .client
                .post(format!(
                    "{CLERK_API_URL}/v1/client/sessions/{sid}/tokens{CLERK_QUERY_VERSION}"
                ))
                .header(COOKIE_HEADER, self.get_header_cookies())
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[("organization_id", "")])
                .send()
                .await
                .map_err(AppError::err)?;

            match res.status() {
                status if status.is_success() => {
                    let data: TokenPayload = res.json().await.map_err(AppError::err)?;
                    *wl = Some(AuthToken::new(data.jwt)?);
                    Ok(())
                }
                status => {
                    if status.is_client_error() {
                        let mut wl = self.session_id.write().unwrap();
                        *wl = None;
                    }
                    let err = self.get_error(res).await?;
                    Err(err)
                }
            }
        })
    }

    pub async fn sign_out(&self) -> Result<(), AppError> {
        self.rt.block_on(async move {
            let res = self
                .client
                .post(format!(
                    "{CLERK_API_URL}/v1/client/sessions{CLERK_QUERY_VERSION}&_method=DELETE"
                ))
                .header(COOKIE_HEADER, self.get_header_cookies())
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .send()
                .await
                .map_err(AppError::err)?;

            let cookies = extract_cookies(res.headers());
            if res.status().is_client_error() {
                let text = res.text().await.map_err(AppError::err)?;
                return Err(AppError::message(format!(
                    "Error deleting session: {}",
                    text
                )));
            }

            let _: ClientPayload = res.json().await.unwrap();

            {
                let mut wl = self.token_spec.write().unwrap();
                *wl = None;

                let mut wl = self.cookies.write().unwrap();
                *wl = cookies;

                let mut wl = self.session_id.write().unwrap();
                *wl = None;
            }

            Ok(())
        })
    }

    async fn get_error(&self, res: reqwest::Response) -> Result<AppError, AppError> {
        let status = res.status();
        let data: ClerkErrorPayload = res.json().await.map_err(AppError::err)?;
        Ok(AppError {
            status: status.as_u16(),
            message: data
                .errors
                .into_iter()
                .fold(String::from(""), |mut acc, err| {
                    acc.push_str(&err.long_message);
                    acc.push_str("; ");
                    acc
                }),
            req_id: "".into(),
        })
    }

    fn update_cookies(&self, cookies: HashMap<String, Cookie>, expires_at: u64) {
        let mut wl = self.cookies.write().unwrap();

        cookies.into_iter().for_each(|(k, v)| {
            wl.insert(
                k,
                Cookie {
                    value: v.value,
                    expires: expires_at,
                },
            );
        });
    }

    fn get_header_cookies(&self) -> String {
        let rl = self.cookies.read().unwrap();
        rl.iter().fold(String::from(""), |mut acc, (k, v)| {
            acc.push_str(k.as_str());
            acc.push_str("=");
            acc.push_str(v.value.as_str());
            acc.push_str(";");
            acc
        })
    }

    pub fn save_data(&self, cx: &mut App) {
        let store = Store::global_mut(cx);
        store.cookies = self.cookies.read().unwrap().clone();
        store.client_id = self.client_id.read().unwrap().clone();
        store.session_id = self.session_id.read().unwrap().clone();
        store.save();
    }
}

fn filter_store_cookies(cookies: HashMap<String, Cookie>) -> HashMap<String, Cookie> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    cookies
        .into_iter()
        .filter(|(_, cookie)| cookie.expires > now)
        .collect()
}

fn extract_cookies(headers: &reqwest::header::HeaderMap) -> HashMap<String, Cookie> {
    let mut cookies = HashMap::new();
    for cookie in headers.get_all(SET_COOKIE_HEADER).into_iter() {
        let cookie_str = cookie.to_str().unwrap();

        if let Some(cookie) = cookie_str.split(';').nth(0) {
            let parts = cookie.split('=').collect::<Vec<_>>();
            cookies.insert(
                parts[0].to_owned(),
                Cookie {
                    value: parts[1].to_owned(),
                    expires: 0,
                },
            );
        }
    }
    cookies
}

// ------------------ Payloads ------------------

#[derive(Debug, Deserialize)]
struct __JwtClaims {
    exp: u64,
}

#[derive(Debug, Deserialize)]
struct __ClerkError {
    // message: String,
    long_message: String,
    // code: String,
}

#[derive(Debug, Deserialize)]
struct ClerkErrorPayload {
    errors: Vec<__ClerkError>,
}

#[derive(Debug, Deserialize)]
struct __ClientParts {
    id: String,
    cookie_expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct ClientPayload {
    response: __ClientParts,
}

#[derive(Debug, Deserialize)]
struct __SignInFirstFactor {
    strategy: String,
    primary: bool,
    email_address_id: String,
}

#[derive(Debug, Deserialize)]
struct __SignInResponse {
    id: String,
    created_session_id: Option<String>,
    supported_first_factors: Option<Vec<__SignInFirstFactor>>,
}

#[derive(Debug, Deserialize)]
struct SignInPayload {
    client: __ClientParts,
    response: __SignInResponse,
}

#[derive(Debug, Deserialize)]
struct TokenPayload {
    jwt: String,
}
