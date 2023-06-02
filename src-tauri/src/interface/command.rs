use std::sync::Arc;
use tauri::State;

use super::{
    error::CommandError,
    module::{Modules, ModulesExt},
};
use crate::{
    domain::{collection::Collection, file::get_lnk_metadatas, Id},
    infrastructure::repositoryimpl::migration::ONEPIECE_COLLECTION_ID,
};

#[tauri::command]
pub async fn greet(name: String) -> String {
    // save_icon_to_png(
    //     "C:\\Users\\ryoha\\Desktop\\ティンクル☆くるせいだーす Remaster\\KuruKuru.exe",
    //     "F:\\workspace\\launcherg\\src-tauri\\image.png",
    // )
    // .unwrap()
    // .await;
    println!("terminated2");
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
) -> anyhow::Result<(), CommandError> {
    let new_elements = modules
        .file_use_case()
        .explore_without_lnk_cache(explore_dir_paths)
        .await?;

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn add_collection_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    explore_dir_paths: Vec<String>,
    with_cache: bool,
    adding_collection_id: Option<i32>,
) -> anyhow::Result<(), CommandError> {
    let new_elements = modules
        .file_use_case()
        .explore_without_lnk_cache(explore_dir_paths)
        .await?;

    // TODO: icon

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements)
        .await?;

    let new_element_ids = new_elements.iter().map(|v| v.id.clone()).collect();

    modules
        .collection_use_case()
        .add_collection_elements(&Id::new(ONEPIECE_COLLECTION_ID), &new_element_ids)
        .await?;
    if let Some(collection_id) = adding_collection_id {
        modules
            .collection_use_case()
            .add_collection_elements(&Id::new(collection_id), &new_element_ids)
            .await?;
    }

    Ok(())
}
