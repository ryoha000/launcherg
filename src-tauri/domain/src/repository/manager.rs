use super::RepositoriesExt;
use futures::future::BoxFuture;

pub trait RepositoryManager<R: RepositoriesExt>: Send + Sync + 'static {
    fn run<'a, T: Send + 'a>(&'a self, f: impl FnOnce(R) -> BoxFuture<'a, anyhow::Result<T>> + Send + 'a) -> BoxFuture<'a, anyhow::Result<T>>;
    fn run_in_transaction<'a, T: Send + 'a>(&'a self, f: impl FnOnce(R) -> BoxFuture<'a, anyhow::Result<T>> + Send + 'a) -> BoxFuture<'a, anyhow::Result<T>>;
}


