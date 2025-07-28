use crate::domain::network::ErogamescapeIDNamePair;

#[cfg_attr(test, mockall::automock)]
pub trait NetworkExplorer {
    async fn get_all_games(&self) -> anyhow::Result<Vec<ErogamescapeIDNamePair>>;
}
