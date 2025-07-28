#[cfg(test)]
mockall::mock! {
    pub ExplorersExtMock {}
    
    impl super::explorerimpl::explorer::ExplorersExt for ExplorersExtMock {
        type FileExplorer = crate::domain::explorer::file::MockFileExplorer;

        fn file_explorer(&self) -> &crate::domain::explorer::file::MockFileExplorer;
    }
}