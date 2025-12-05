use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::RwLock, AppHandle};
use tauri_plugin_http::reqwest;
use tauri_plugin_store::StoreExt;

use crate::app::{IpcError, IpcResult};

const CLERK_API_URL: &str = "https://clerk.somachron.shank03.com";
const CLERK_QUERY_VERSION: &str = "?__clerk_api_version=2025-11-10";
const CLERK_STRATEGY: &str = "email_code";

const SET_COOKIE_HEADER: &str = "Set-Cookie";
const COOKIE_HEADER: &str = "Cookie";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const FORM_DATA_TYPE: &str = "application/x-www-form-urlencoded";

const APP_SETTINGS: &str = "app.json";
const COOKIE_SETTING: &str = "cookies";
const CLIENT_SETTING: &str = "client_id";
const SESSION_SETTING: &str = "session_id";

#[derive(Clone)]
struct AuthToken {
    token: String,
    exp: u64, // secs
}

impl AuthToken {
    fn new(token: String) -> IpcResult<Self> {
        let payload = match token.split('.').nth(1) {
            Some(p) => base64::prelude::BASE64_STANDARD_NO_PAD
                .decode(p)
                .map_err(IpcError::err)?,
            None => return Err(IpcError::message("Invalid token")),
        };

        let __JwtClaims { exp } = serde_json::from_slice(&payload).map_err(IpcError::err)?;

        Ok(Self { token, exp })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cookie {
    pub value: String,
    pub expires: u64,
}

pub type AuthState = Arc<Auth>;

pub struct Auth {
    app: AppHandle,
    client: reqwest::Client,
    cookies: RwLock<HashMap<String, Cookie>>,

    client_id: RwLock<Option<String>>,
    sign_in_id: RwLock<Option<String>>,
    session_id: RwLock<Option<String>>,

    token_spec: RwLock<Option<AuthToken>>,
}

impl Auth {
    pub fn init(app: AppHandle) -> AuthState {
        let client = reqwest::ClientBuilder::new()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.append("Host", "clerk.somachron.shank03.com".parse().unwrap());
                headers.append("Origin", "https://somachron.shank03.com".parse().unwrap());
                headers.append("Referer", "https://somachron.shank03.com".parse().unwrap());
                headers
            })
            .build()
            .unwrap();

        let store = app.store(APP_SETTINGS).expect("Failed to create store");

        let cookies = store
            .get(COOKIE_SETTING)
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(HashMap::new());
        let client_id = store
            .get(CLIENT_SETTING)
            .and_then(|v| serde_json::from_value(v).ok());
        let session_id = store
            .get(SESSION_SETTING)
            .and_then(|v| serde_json::from_value(v).ok());

        Arc::new(Self {
            app,
            client,
            cookies: RwLock::new(cookies),
            client_id: RwLock::new(client_id),
            sign_in_id: RwLock::new(None),
            session_id: RwLock::new(session_id),
            token_spec: RwLock::new(None),
        })
    }

    pub async fn setup_client(&self) -> IpcResult<()> {
        {
            let gc = self.client_id.read().await;
            if let Some(_) = &*gc {
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
            self.cookies
                .write()
                .await
                .extend(extract_cookies(res.headers()));
        }

        if res.status().is_client_error() {
            let text = res.text().await.map_err(IpcError::err)?;
            return Err(IpcError::message(format!("Error setting up env: {}", text)));
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
            let text = res.text().await.map_err(IpcError::err)?;
            return Err(IpcError::message(format!("Error setting up env: {}", text)));
        }

        let data: ClientPayload = res.json().await.unwrap();

        {
            let mut g_cid = self.client_id.write().await;
            *g_cid = Some(data.response.id);

            self.update_cookies(cookies, data.response.cookie_expires_at)
                .await;
        }

        Ok(())
    }

    pub async fn sign_in(&self, email: &str) -> IpcResult<String> {
        {
            let g_cid = self.client_id.read().await;
            if g_cid.is_none() {
                return Err(IpcError::message("No client id"));
            }
        };

        let res = self
            .client
            .post(format!(
                "{CLERK_API_URL}/v1/client/sign_ins{CLERK_QUERY_VERSION}"
            ))
            .header(COOKIE_HEADER, self.get_header_cookies().await)
            .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
            .form(&[("identifier", email)])
            .send()
            .await
            .map_err(IpcError::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcError::err)?;
            let idn = data
                .response
                .supported_first_factors
                .and_then(|factors| {
                    factors
                        .into_iter()
                        .find(|f| f.primary && f.strategy.as_str().cmp(CLERK_STRATEGY).is_eq())
                })
                .ok_or(IpcError::message("No primary factor found"))?;

            {
                let mut guard = self.sign_in_id.write().await;
                *guard = Some(data.response.id);
            }
            self.update_cookies(cookies, data.client.cookie_expires_at)
                .await;

            Ok(idn.email_address_id)
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    pub async fn prepare_first_factor(&self, email_address_id: &str) -> IpcResult<()> {
        let sia = {
            let g_sia = self.sign_in_id.read().await;
            match &*g_sia {
                Some(s) => s.clone(),
                None => return Err(IpcError::message("No sign in instance")),
            }
        };

        let res = self
                .client
                .post(format!(
                    "{CLERK_API_URL}/v1/client/sign_ins/{sia}/prepare_first_factor{CLERK_QUERY_VERSION}"
                ))
                .header(COOKIE_HEADER, self.get_header_cookies().await)
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[
                    ("email_address_id", email_address_id),
                    ("strategy", CLERK_STRATEGY),
                ])
                .send()
                .await
                .map_err(IpcError::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcError::err)?;
            self.update_cookies(cookies, data.client.cookie_expires_at)
                .await;
            Ok(())
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    pub async fn attempt_first_factor(&self, code: &str) -> IpcResult<()> {
        let sia = {
            let g_sia = self.sign_in_id.read().await;
            match &*g_sia {
                Some(s) => s.clone(),
                None => return Err(IpcError::message("No sign in instance")),
            }
        };

        let res = self
                .client
                .post(format!(
                    "{CLERK_API_URL}/v1/client/sign_ins/{sia}/attempt_first_factor{CLERK_QUERY_VERSION}"
                ))
                .header(COOKIE_HEADER, self.get_header_cookies().await)
                .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
                .form(&[("code", code), ("strategy", CLERK_STRATEGY)])
                .send()
                .await
                .map_err(IpcError::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcError::err)?;

            let session_id = data
                .response
                .created_session_id
                .ok_or(IpcError::message("No created session ID"))?;

            {
                let mut sg = self.session_id.write().await;
                *sg = Some(session_id);
            }

            self.update_cookies(cookies, data.client.cookie_expires_at)
                .await;
            Ok(())
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    pub async fn get_token(&self) -> IpcResult<String> {
        let spec = {
            let gt = self.token_spec.read().await;
            match &*gt {
                Some(token) => token.clone(),
                None => return Err(IpcError::message("No token !")),
            }
        };

        let current_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .and_then(|v| Ok(v.as_secs()))
            .unwrap_or(1);

        if current_secs > spec.exp {
            println!(
                "Fetching token: current: {}, exp: {}",
                current_secs, spec.exp
            );
            self.fetch_token().await?;
        }

        {
            let gt = self.token_spec.read().await;
            match &*gt {
                Some(token) => Ok(token.token.clone()),
                None => Err(IpcError::message("No token !")),
            }
        }
    }

    pub async fn fetch_token(&self) -> IpcResult<()> {
        let sid = {
            let g_sid = self.session_id.read().await;
            match &*g_sid {
                Some(s) => s.clone(),
                None => return Err(IpcError::message("No session id")),
            }
        };

        let res = self
            .client
            .post(format!(
                "{CLERK_API_URL}/v1/client/sessions/{sid}/tokens{CLERK_QUERY_VERSION}"
            ))
            .header(COOKIE_HEADER, self.get_header_cookies().await)
            .header(CONTENT_TYPE_HEADER, FORM_DATA_TYPE)
            .form(&[("organization_id", "")])
            .send()
            .await
            .map_err(IpcError::err)?;

        if res.status().is_success() {
            let data: TokenPayload = res.json().await.map_err(IpcError::err)?;

            {
                let mut gt = self.token_spec.write().await;
                *gt = Some(AuthToken::new(data.jwt)?);
            }
            Ok(())
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    async fn get_header_cookies(&self) -> String {
        let rl = self.cookies.read().await;
        rl.iter().fold(String::from(""), |mut acc, (k, v)| {
            acc.push_str(k.as_str());
            acc.push_str("=");
            acc.push_str(v.value.as_str());
            acc.push_str(";");
            acc
        })
    }

    async fn get_error(&self, res: reqwest::Response) -> IpcResult<IpcError> {
        let status = res.status();
        let data: ClerkErrorPayload = res.json().await.map_err(IpcError::err)?;
        Ok(IpcError {
            status: status.as_u16(),
            message: data
                .errors
                .into_iter()
                .fold(String::from(""), |mut acc, err| {
                    acc.push_str(&err.long_message);
                    acc.push_str(" ");
                    acc
                }),
            req_id: "".into(),
        })
    }

    async fn update_cookies(&self, cookies: HashMap<String, Cookie>, expires_at: u64) {
        let mut cg = self.cookies.write().await;
        cookies.into_iter().for_each(|(k, v)| {
            cg.insert(
                k,
                Cookie {
                    value: v.value,
                    expires: expires_at,
                },
            );
        });
    }

    pub async fn save_data(&self) {
        let store = self.app.store(APP_SETTINGS).expect("Failed to open store");
        store.set(
            COOKIE_SETTING,
            serde_json::to_value(&*self.cookies.read().await).unwrap(),
        );
        store.set(
            CLIENT_SETTING,
            serde_json::to_value(&*self.client_id.read().await).unwrap(),
        );
        store.set(
            SESSION_SETTING,
            serde_json::to_value(&*self.session_id.read().await).unwrap(),
        );
    }
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
