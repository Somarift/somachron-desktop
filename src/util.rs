use std::future::Future;

pub fn tokio_rt<Fut, R, E>(f: Fut) -> Result<R, E>
where
    Fut: Future<Output = Result<R, E>>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}
