pub trait MapAsync<T, E> {
    async fn map_async<U>(self, f: impl AsyncFnOnce(T) -> Result<U, E> + 'static) -> Result<U, E>;
}

impl<T, E> MapAsync<T, E> for Result<T, E> {
    async fn map_async<U>(self, f: impl AsyncFnOnce(T) -> Result<U, E> + 'static) -> Result<U, E> {
        match self {
            Ok(ok) => f(ok).await,
            Err(err) => Err(err),
        }
    }
}
