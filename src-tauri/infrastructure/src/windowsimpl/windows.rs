use derive_new::new;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::windowsimpl::proctail::ProcTailImpl;
use crate::windowsimpl::proctail_manager::ProcTailManager;
use crate::windowsimpl::shell_link::ShellLinkImpl;
use domain::service::save_path_resolver::DirsSavePathResolver;
use domain::{process::Process, windows::WindowsExt};

#[derive(new)]
pub struct WindowsImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Windows {
    process: WindowsImpl<Process>,
    proctail: ProcTailImpl,
    proctail_manager: ProcTailManager<DirsSavePathResolver>,
    shell_link: ShellLinkImpl,
}

impl WindowsExt for Windows {
    type ProcessWindows = WindowsImpl<Process>;
    type ProcTail = ProcTailImpl;
    type ProcTailManager = ProcTailManager<DirsSavePathResolver>;
    type ShellLink = ShellLinkImpl;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }

    fn proctail(&self) -> &Self::ProcTail {
        &self.proctail
    }

    fn proctail_manager(&self) -> &Self::ProcTailManager {
        &self.proctail_manager
    }

    fn shell_link(&self) -> &Self::ShellLink {
        &self.shell_link
    }
}

impl Windows {
    pub fn new() -> Self {
        let process = WindowsImpl::new();
        let proctail = ProcTailImpl::new();
        let proctail_manager = ProcTailManager::new(Arc::new(DirsSavePathResolver::default()));
        let shell_link = ShellLinkImpl::new();

        Self {
            process,
            proctail,
            proctail_manager,
            shell_link,
        }
    }
}
