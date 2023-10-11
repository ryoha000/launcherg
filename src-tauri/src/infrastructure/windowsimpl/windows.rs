use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{process::Process, windows::process::ProcessWindows};

#[derive(new)]
pub struct WindowsImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Windows {
    process: WindowsImpl<Process>,
}
pub trait WindowsExt {
    type ProcessWindows: ProcessWindows;

    fn process(&self) -> &Self::ProcessWindows;
}

impl WindowsExt for Windows {
    type ProcessWindows = WindowsImpl<Process>;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }
}

impl Windows {
    pub fn new() -> Self {
        let process = WindowsImpl::new();

        Self { process }
    }
}
