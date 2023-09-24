use std::sync::Arc;

use derive_new::new;

use crate::{
    domain::{explorer::network::NetworkExplorer, network::ErogamescapeIDNamePair},
    infrastructure::explorerimpl::explorer::ExplorersExt,
};

#[derive(new)]
pub struct NetworkUseCase<R: ExplorersExt> {
    explorers: Arc<R>,
}

impl<R: ExplorersExt> NetworkUseCase<R> {
    pub async fn get_all_games(&self) -> anyhow::Result<Vec<ErogamescapeIDNamePair>> {
        Ok(self.explorers.network_explorer().get_all_games().await?)
    }
}
