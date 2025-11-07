use std::sync::OnceLock;

pub mod api;
pub mod web_client;
pub use web_client::*;

static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn tokio_rt<Fut, R, E>(f: Fut) -> Result<R, E>
where
    Fut: Future<Output = Result<R, E>>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}

pub fn get_tokio_rt() -> tokio::runtime::Handle {
    tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
        let rt = TOKIO_RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio bruh")
        });
        rt.handle().clone()
    })
}
