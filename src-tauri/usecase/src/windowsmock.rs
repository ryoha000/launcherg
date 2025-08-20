#[cfg(any(test, feature = "mocks"))]
mockall::mock! {
    pub ProcTailManagerTrait {}

    impl ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerTrait for ProcTailManagerTrait {
        async fn get_status(&self) -> Result<
            ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerStatus,
            ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError
        >;
        async fn get_latest_version(&self) -> Result<
            ::infrastructure::windowsimpl::proctail_manager::ProcTailVersion,
            ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError
        >;
        async fn is_update_available(&self) -> Result<bool, ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError>;
        async fn download_and_install(
            &self,
            version: &::infrastructure::windowsimpl::proctail_manager::ProcTailVersion,
        ) -> Result<(), ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError>;
        async fn start_proctail(&self) -> Result<(), ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError>;
        async fn stop_proctail(&self) -> Result<(), ::infrastructure::windowsimpl::proctail_manager::ProcTailManagerError>;
        async fn is_running(&self) -> bool;
    }
}

#[cfg(any(test, feature = "mocks"))]
mockall::mock! {
    pub WindowsExtMock {}

    impl ::infrastructure::windowsimpl::windows::WindowsExt for WindowsExtMock {
        type ProcessWindows = crate::domain::windows::process::MockProcessWindows;
        type ProcTail = crate::domain::windows::proctail::MockProcTail;
        type ProcTailManager = MockProcTailManagerTrait;

        fn process(&self) -> &crate::domain::windows::process::MockProcessWindows;
        fn proctail(&self) -> &crate::domain::windows::proctail::MockProcTail;
        fn proctail_manager(&self) -> &MockProcTailManagerTrait;
    }
}


