use std::sync::Arc;

use derive_new::new;

use crate::infrastructure::{
    repositoryimpl::repository::Repositories, windowsimpl::windows::Windows,
};

#[derive(new)]
pub struct Modules {
    repository: Arc<Repositories>,
    windows: Arc<Windows>,
}

impl Modules {
    pub fn repository(&self) -> Arc<Repositories> {
        self.repository.clone()
    }
    pub fn windows(&self) -> Arc<Windows> {
        self.windows.clone()
    }
}
