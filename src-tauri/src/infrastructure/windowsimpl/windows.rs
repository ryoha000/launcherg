use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{process::Process, windows::process::ProcessWindows, windows::proctail::ProcTail};
use crate::infrastructure::windowsimpl::proctail::ProcTailImpl;

#[derive(new)]
pub struct WindowsImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Windows {
    process: WindowsImpl<Process>,
    proctail: ProcTailImpl,
}
pub trait WindowsExt {
    type ProcessWindows: ProcessWindows;
    type ProcTail: ProcTail;

    fn process(&self) -> &Self::ProcessWindows;
    fn proctail(&self) -> &Self::ProcTail;
}

impl WindowsExt for Windows {
    type ProcessWindows = WindowsImpl<Process>;
    type ProcTail = ProcTailImpl;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }

    fn proctail(&self) -> &Self::ProcTail {
        &self.proctail
    }
}

impl Windows {
    pub fn new() -> Self {
        let process = WindowsImpl::new();
        let proctail = ProcTailImpl::new();

        Self { process, proctail }
    }
}
