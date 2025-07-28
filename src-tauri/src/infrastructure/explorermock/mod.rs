#[cfg(test)]
mockall::mock! {
    pub ExplorersExtMock {}
    
    impl super::explorerimpl::explorer::ExplorersExt for ExplorersExtMock {
        type FileExplorer = crate::domain::explorer::file::MockFileExplorer;
        type NetworkExplorer = crate::domain::explorer::network::MockNetworkExplorer;

        fn file_explorer(&self) -> &crate::domain::explorer::file::MockFileExplorer;
        fn network_explorer(&self) -> &crate::domain::explorer::network::MockNetworkExplorer;
    }
}