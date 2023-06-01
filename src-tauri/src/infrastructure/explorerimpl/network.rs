use async_trait::async_trait;

use super::explorer::ExplorerImpl;
use crate::domain::{
    explorer::network::NetworkExplorer,
    network::{ErogamescapeIDNamePair, NetWork},
};

#[async_trait]
impl NetworkExplorer for ExplorerImpl<NetWork> {
    async fn get_all_games(&self) -> anyhow::Result<Vec<ErogamescapeIDNamePair>> {
        Ok(reqwest::get(
            "https://raw.githubusercontent.com/ryoha000/launcherg/main/script/all_games.json",
        )
        .await?
        .json::<Vec<ErogamescapeIDNamePair>>()
        .await?)
    }
}
