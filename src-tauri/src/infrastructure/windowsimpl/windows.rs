use derive_new::new;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::domain::{
    process::Process, windows::process::ProcessWindows, windows::proctail::ProcTail,
};
use crate::infrastructure::windowsimpl::proctail::ProcTailImpl;
use crate::infrastructure::windowsimpl::proctail_manager::{
    AppHandleProcTailManager, ProcTailManagerTrait,
};
use tauri::AppHandle;

#[derive(new)]
pub struct WindowsImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Windows {
    process: WindowsImpl<Process>,
    proctail: ProcTailImpl,
    proctail_manager: AppHandleProcTailManager,
}
pub trait WindowsExt {
    type ProcessWindows: ProcessWindows;
    type ProcTail: ProcTail;
    type ProcTailManager: ProcTailManagerTrait;

    fn process(&self) -> &Self::ProcessWindows;
    fn proctail(&self) -> &Self::ProcTail;
    fn proctail_manager(&self) -> &Self::ProcTailManager;
}


impl WindowsExt for Windows {
    type ProcessWindows = WindowsImpl<Process>;
    type ProcTail = ProcTailImpl;
    type ProcTailManager = AppHandleProcTailManager;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }

    fn proctail(&self) -> &Self::ProcTail {
        &self.proctail
    }

    fn proctail_manager(&self) -> &Self::ProcTailManager {
        &self.proctail_manager
    }
}

impl Windows {
    pub fn new(app_handle: Arc<AppHandle>) -> Self {
        let process = WindowsImpl::new();
        let proctail = ProcTailImpl::new();
        let proctail_manager = AppHandleProcTailManager::new(app_handle);

        Self {
            process,
            proctail,
            proctail_manager,
        }
    }
}
