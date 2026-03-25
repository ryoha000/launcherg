use derive_new::new;
use std::marker::PhantomData;

use crate::windowsimpl::shell_link::ShellLinkImpl;
use domain::{process::Process, windows::WindowsExt};

#[derive(new)]
pub struct WindowsImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Windows {
    process: WindowsImpl<Process>,
    shell_link: ShellLinkImpl,
}

impl WindowsExt for Windows {
    type ProcessWindows = WindowsImpl<Process>;
    type ShellLink = ShellLinkImpl;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }

    fn shell_link(&self) -> &Self::ShellLink {
        &self.shell_link
    }
}

impl Windows {
    pub fn new() -> Self {
        let process = WindowsImpl::new();
        let shell_link = ShellLinkImpl::new();

        Self {
            process,
            shell_link,
        }
    }
}
