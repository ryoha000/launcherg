use async_trait::async_trait;

use crate::domain::network::ErogamescapeIDNamePair;

#[async_trait]
pub trait NetworkExplorer {
    async fn get_all_games(&self) -> anyhow::Result<Vec<ErogamescapeIDNamePair>>;
}
