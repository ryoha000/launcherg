#[cfg(test)]
mockall::mock! {
    pub WindowsExtMock {}
    
    impl super::windowsimpl::windows::WindowsExt for WindowsExtMock {
        type ProcessWindows = crate::domain::windows::process::MockProcessWindows;
        type ProcTail = crate::domain::windows::proctail::MockProcTail;
        type ProcTailManager = crate::infrastructure::windowsimpl::proctail_manager::MockProcTailManagerTrait;

        fn process(&self) -> &crate::domain::windows::process::MockProcessWindows;
        fn proctail(&self) -> &crate::domain::windows::proctail::MockProcTail;
        fn proctail_manager(&self) -> &crate::infrastructure::windowsimpl::proctail_manager::MockProcTailManagerTrait;
    }
}