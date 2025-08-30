use anyhow::Result;
use futures::future::BoxFuture;

use super::RepositoriesExt;

#[trait_variant::make(Send)]
pub trait TransactionRepository: RepositoriesExt {
    async fn with_transaction<F, R>(&mut self, f: F) -> Result<R>
    where
        for<'cx> F: FnOnce(&'cx mut Self) -> BoxFuture<'cx, Result<R>> + Send,
        R: Send;
}