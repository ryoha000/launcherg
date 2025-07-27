use crate::domain::{
    windows::process::ProcessWindows, windows::proctail::ProcTail,
};
use crate::infrastructure::windowsimpl::proctail_manager::ProcTailManagerTrait;

use super::windowsimpl::windows::WindowsExt;

#[cfg(test)]
mockall::mock! {
    pub WindowsExtMock {}
    impl WindowsExt for WindowsExtMock {
        type ProcessWindows = crate::domain::windows::process::MockProcessWindows;
        type ProcTail = crate::domain::windows::proctail::MockProcTail;
        type ProcTailManager = crate::infrastructure::windowsimpl::proctail_manager::MockProcTailManagerTrait;

        fn process(&self) -> &Self::ProcessWindows;
        fn proctail(&self) -> &Self::ProcTail;
        fn proctail_manager(&self) -> &Self::ProcTailManager;
    }
}