use std::sync::Arc;
use tauri::State;

use super::{
    error::CommandError,
    module::{Modules, ModulesExt},
};
use crate::domain::{collection::Collection, network::ErogamescapeIDNamePair};

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

#[tauri::command]
pub async fn explore(
    modules: State<'_, Arc<Modules>>,
    explore_dir_paths: Vec<String>,
    with_cache: bool,
) -> anyhow::Result<Vec<(ErogamescapeIDNamePair, String)>, CommandError> {
    Ok(modules
        .file_use_case()
        .explore_without_lnk_cache(explore_dir_paths)
        .await?)
}
