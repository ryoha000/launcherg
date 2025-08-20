pub mod process;
pub mod proctail;
pub mod proctail_manager;

pub trait WindowsExt {
    type ProcessWindows: process::ProcessWindows;
    type ProcTail: proctail::ProcTail;
    type ProcTailManager: proctail_manager::ProcTailManagerTrait;

    fn process(&self) -> &Self::ProcessWindows;
    fn proctail(&self) -> &Self::ProcTail;
    fn proctail_manager(&self) -> &Self::ProcTailManager;
}
