use derive_new::new;
use std::marker::PhantomData;
use std::sync::Arc;

use domain::{
    process::Process, windows::WindowsExt,
};
use crate::windowsimpl::proctail::ProcTailImpl;
use crate::windowsimpl::proctail_manager::AppHandleProcTailManager;
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
