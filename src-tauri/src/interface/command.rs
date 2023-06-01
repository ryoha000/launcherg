use std::sync::Arc;
use tauri::State;

use super::{
    error::CommandError,
    module::{Modules, ModulesExt},
};
use crate::domain::collection::Collection;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn get_all_collections(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<Collection>, CommandError> {
    Ok(modules.collection_use_case().get_all_collections().await?)
}
