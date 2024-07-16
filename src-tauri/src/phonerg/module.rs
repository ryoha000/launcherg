use std::sync::{Arc, Mutex};

use derive_new::new;
use tauri::AppHandle;

use crate::infrastructure::{
    explorerimpl::explorer::Explorers, repositoryimpl::repository::Repositories,
    windowsimpl::windows::Windows,
};

use super::handler::models::current::Current;

#[derive(new)]
pub struct Modules {
    current: Mutex<Option<Current>>,
    handle: Arc<AppHandle>,

    repository: Arc<Repositories>,
    windows: Arc<Windows>,
    explorers: Arc<Explorers>,
}

impl Modules {
    pub fn current(&self) -> &Mutex<Option<Current>> {
        &self.current
    }
    pub fn handle(&self) -> Arc<AppHandle> {
        self.handle.clone()
    }

    pub fn repository(&self) -> Arc<Repositories> {
        self.repository.clone()
    }
    pub fn windows(&self) -> Arc<Windows> {
        self.windows.clone()
    }
    pub fn explorers(&self) -> Arc<Explorers> {
        self.explorers.clone()
    }
}
