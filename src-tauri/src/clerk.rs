use std::{collections::BTreeMap, sync::Arc};

use serde::Deserialize;
use tokio::sync::RwLock;

use crate::commands::IpcResult;

const CLERK_API_URL: &str = "https://clerk.somachron.shank03.com";
const CLERK_QUERY_VERSION: &str = "?__clerk_api_version=2025-04-10&_clerk_js_version=5.80.0";
const CLERK_STRATEGY: &str = "email_code";

const SET_COOKIE_HEADER: &str = "Set-Cookie";
const COOKIE_HEADER: &str = "Cookie";
const CONTENT_TYPE_HEADER: &str = "Content-Type";
const FORM_DATA_TYPE: &str = "application/x-www-form-urlencoded";

pub type ClerkState = Arc<Clerk>;

#[derive(Debug)]
pub struct Cookie {
    value: String,
    expires: u64,
}

pub struct Clerk {
    client: reqwest::Client,
    cookies: Arc<RwLock<BTreeMap<String, Cookie>>>,

    client_id: Arc<RwLock<Option<String>>>,
    sign_in_id: Arc<RwLock<Option<String>>>,
    session_id: Arc<RwLock<Option<String>>>,
    token: Arc<RwLock<Option<String>>>,
}

impl Clerk {
    pub fn init() -> Arc<Self> {
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

        Arc::new(Self {
            client,
            cookies: Arc::new(RwLock::new(BTreeMap::new())),
            client_id: Arc::new(RwLock::new(None)),
            sign_in_id: Arc::new(RwLock::new(None)),
            session_id: Arc::new(RwLock::new(None)),
            token: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn setup_client(&self) -> Result<(), IpcResult> {
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
            let text = res.text().await.map_err(IpcResult::err)?;
            return Err(IpcResult::message(format!(
                "Error setting up env: {}",
                text
            )));
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
            let text = res.text().await.map_err(IpcResult::err)?;
            return Err(IpcResult::message(format!(
                "Error setting up env: {}",
                text
            )));
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

    pub async fn sign_in(&self, email: &str) -> Result<String, IpcResult> {
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
            .map_err(IpcResult::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcResult::err)?;
            let idn = data
                .response
                .supported_first_factors
                .and_then(|factors| {
                    factors
                        .into_iter()
                        .find(|f| f.primary && f.strategy.as_str().cmp(CLERK_STRATEGY).is_eq())
                })
                .ok_or(IpcResult::message("No primary factor found"))?;

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

    pub async fn prepare_first_factor(&self, email_address_id: &str) -> Result<(), IpcResult> {
        let g_sia = self.sign_in_id.read().await;
        let sia = match &*g_sia {
            Some(s) => s.clone(),
            None => return Err(IpcResult::message("No sign in instance")),
        };
        drop(g_sia);

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
            .map_err(IpcResult::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcResult::err)?;
            self.update_cookies(cookies, data.client.cookie_expires_at)
                .await;
            Ok(())
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    pub async fn attempt_first_factor(&self, code: &str) -> Result<(), IpcResult> {
        let g_sia = self.sign_in_id.read().await;
        let sia = match &*g_sia {
            Some(s) => s.clone(),
            None => return Err(IpcResult::message("No sign in instance")),
        };
        drop(g_sia);

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
            .map_err(IpcResult::err)?;

        let cookies = extract_cookies(res.headers());

        if res.status().is_success() {
            let data: SignInPayload = res.json().await.map_err(IpcResult::err)?;

            let session_id = data
                .response
                .created_session_id
                .ok_or(IpcResult::message("No created session ID"))?;

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

    pub async fn get_token(&self) -> Result<(), IpcResult> {
        let g_sid = self.session_id.read().await;
        let sid = match &*g_sid {
            Some(s) => s.clone(),
            None => return Err(IpcResult::message("No sign in instance")),
        };
        drop(g_sid);

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
            .map_err(IpcResult::err)?;

        if res.status().is_success() {
            let data: TokenPayload = res.json().await.map_err(IpcResult::err)?;

            {
                let mut gt = self.token.write().await;
                *gt = Some(data.jwt);
            }
            Ok(())
        } else {
            let err = self.get_error(res).await?;
            Err(err)
        }
    }

    async fn get_error(&self, res: reqwest::Response) -> Result<IpcResult, IpcResult> {
        let status = res.status();
        let data: ClerkErrorPayload = res.json().await.map_err(IpcResult::err)?;
        Ok(IpcResult {
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

    async fn update_cookies(&self, cookies: BTreeMap<String, Cookie>, expires_at: u64) {
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

    async fn get_header_cookies(&self) -> String {
        self.cookies
            .read()
            .await
            .iter()
            .fold(String::from(""), |mut acc, (k, v)| {
                acc.push_str(k.as_str());
                acc.push_str("=");
                acc.push_str(v.value.as_str());
                acc.push_str(";");
                acc
            })
    }
}

fn extract_cookies(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, Cookie> {
    let mut cookies = BTreeMap::new();
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
