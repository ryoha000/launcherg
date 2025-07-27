use crate::domain::{
    explorer::{file::FileExplorer, network::NetworkExplorer},
};

use super::explorerimpl::explorer::ExplorersExt;

#[cfg(test)]
mockall::mock! {
    pub ExplorersExtMock {}
    impl ExplorersExt for ExplorersExtMock {
        type FileExplorer = crate::domain::explorer::file::MockFileExplorer;
        type NetworkExplorer = crate::domain::explorer::network::MockNetworkExplorer;

        fn file_explorer(&self) -> &Self::FileExplorer;
        fn network_explorer(&self) -> &Self::NetworkExplorer;
    }
}