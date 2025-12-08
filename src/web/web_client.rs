use std::{any::type_name, pin::Pin, sync::LazyLock, task::Poll};

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use futures::FutureExt;
use gpui::http_client::http;
use reqwest::header::HeaderValue;

use crate::util;

static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .expect("tokio bruh")
});

pub struct WebClient {
    client: reqwest::Client,
    user_agent: HeaderValue,
}

impl WebClient {
    pub fn new() -> Self {
        let user_agent = make_user_agent();

        let client = make_http_client();
        Self { client, user_agent }
    }
}

pub fn make_user_agent() -> reqwest::header::HeaderValue {
    reqwest::header::HeaderValue::from_str(
        format!(
            "Somachron-Desktop/0.1.0 ({} - {})",
            util::os_name(),
            util::os_version()
        )
        .as_str(),
    )
    .expect("Failed to create user-agent header")
}

pub(super) fn make_http_client() -> reqwest::Client {
    let user_agent = make_user_agent();

    reqwest::Client::builder()
        .default_headers({
            let mut map = reqwest::header::HeaderMap::new();
            map.insert(http::header::USER_AGENT, user_agent.clone());
            map
        })
        .use_rustls_tls()
        .tcp_nodelay(true)
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest bruh")
}

impl gpui::http_client::HttpClient for WebClient {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn user_agent(&self) -> Option<&reqwest::header::HeaderValue> {
        Some(&self.user_agent)
    }

    fn send(
        &self,
        req: gpui::http_client::http::Request<gpui::http_client::AsyncBody>,
    ) -> futures::future::BoxFuture<
        'static,
        anyhow::Result<gpui::http_client::Response<gpui::http_client::AsyncBody>>,
    > {
        let (parts, body) = req.into_parts();

        let request = self
            .client
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .body(match body.0 {
                gpui::http_client::Inner::Empty => reqwest::Body::default(),
                gpui::http_client::Inner::Bytes(cursor) => cursor.into_inner().into(),
                gpui::http_client::Inner::AsyncReader(pin) => {
                    reqwest::Body::wrap_stream(FuturesStreamReader::new(pin))
                }
            });

        // TODO: redirect policies are supposed to be applied on client instead of request
        // if let Some(redirect) = parts.extensions.get::<gpui::http_client::RedirectPolicy>() {
        //     request = request.redirect_policy(match redirect_policy {
        //         RedirectPolicy::NoFollow => redirect::Policy::none(),
        //         RedirectPolicy::FollowLimit(limit) => redirect::Policy::limited(*limit as usize),
        //         RedirectPolicy::FollowAll => redirect::Policy::limited(100),
        //     });
        // }

        async move {
            TOKIO_RUNTIME
                .spawn(async move {
                    let response = request.send().await?;

                    let headers = response.headers().clone();
                    let mut builder = http::Response::builder()
                        .status(response.status().as_u16())
                        .version(response.version());
                    *builder.headers_mut().unwrap() = headers;

                    let bytes = response.bytes().await?;
                    let body = gpui::http_client::AsyncBody::from_bytes(bytes);

                    builder.body(body).map_err(|e| anyhow!(e))
                })
                .await?
        }
        .boxed()
    }

    fn proxy(&self) -> Option<&reqwest::Url> {
        todo!()
    }
}

pub type GpuiStreamReader = Pin<Box<dyn futures::AsyncRead + Send + Sync>>;

pub struct FuturesStreamReader {
    reader: Option<GpuiStreamReader>,
    buf: BytesMut,
    capacity: usize,
}

impl FuturesStreamReader {
    fn new(reader: GpuiStreamReader) -> Self {
        Self {
            reader: Some(reader),
            buf: BytesMut::new(),
            capacity: 4096,
        }
    }
}

impl futures::Stream for FuturesStreamReader {
    type Item = std::io::Result<bytes::Bytes>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.as_mut();

        let Some(mut reader) = this.reader.take() else {
            return Poll::Ready(None);
        };

        if this.buf.capacity() == 0 {
            let cap = this.capacity;
            this.buf.reserve(cap);
        }

        match poll_read_buf(&mut reader, cx, &mut this.buf) {
            Poll::Ready(result) => match result {
                Ok(n) => {
                    if n == 0 {
                        self.reader = None;
                        Poll::Ready(None)
                    } else {
                        let chunk = this.buf.split();
                        self.reader = Some(reader);
                        Poll::Ready(Some(Ok(chunk.freeze())))
                    }
                }
                Err(err) => {
                    self.reader = None;
                    Poll::Ready(Some(Err(err)))
                }
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Implementation from <https://docs.rs/tokio-util/latest/src/tokio_util/util/poll_buf.rs.html#47>
/// Specialized for this use case
pub fn poll_read_buf(
    io: &mut GpuiStreamReader,
    cx: &mut std::task::Context<'_>,
    buf: &mut BytesMut,
) -> Poll<std::io::Result<usize>> {
    if !buf.has_remaining_mut() {
        return Poll::Ready(Ok(0));
    }

    let n = {
        let dst = buf.chunk_mut();

        // Safety: `chunk_mut()` returns a `&mut UninitSlice`, and `UninitSlice` is a
        // transparent wrapper around `[MaybeUninit<u8>]`.
        let dst = unsafe { dst.as_uninit_slice_mut() };
        let mut buf = tokio::io::ReadBuf::uninit(dst);
        let ptr = buf.filled().as_ptr();

        let io_pin = io.as_mut();
        std::task::ready!(io_pin.poll_read(cx, buf.initialize_unfilled())?);

        // Ensure the pointer does not change from under us
        assert_eq!(ptr, buf.filled().as_ptr());
        buf.filled().len()
    };

    // Safety: This is guaranteed to be the number of initialized (and read)
    // bytes due to the invariants provided by `ReadBuf::filled`.
    unsafe {
        buf.advance_mut(n);
    }

    Poll::Ready(Ok(n))
}
