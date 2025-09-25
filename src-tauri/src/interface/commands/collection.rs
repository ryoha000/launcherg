use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::interface::error::CommandError;
use crate::interface::models::all_game_cache::AllGameCacheOne;
use crate::interface::module::{Modules, ModulesExt};

use domain::windows::shell_link::ShellLink as _;
use domain::windows::WindowsExt as _;

use crate::domain::collection::ScannedGameElement;
use crate::domain::file::get_file_created_at_sync;

#[tauri::command]
pub async fn upsert_collection_element(
    _handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    exe_path: Option<String>,
    lnk_path: Option<String>,
    game_cache: AllGameCacheOne,
) -> anyhow::Result<(), CommandError> {
    let _install_at;
    if let Some(path) = exe_path.clone() {
        _install_at = get_file_created_at_sync(&path);
    } else if let Some(path) = lnk_path.clone() {
        let windows = crate::infrastructure::windowsimpl::windows::Windows::new();
        let metadatas = windows.shell_link().get_lnk_metadatas(vec![path.clone()])?;
        let metadata = metadatas
            .get(&path)
            .ok_or(anyhow::anyhow!("metadata cannot get"))?;
        println!(
            "metadata.path: {}, metadata.icon: {}",
            metadata.path, metadata.icon
        );
        _install_at = get_file_created_at_sync(&metadata.path);
    } else {
        _install_at = None;
    }

    let egs_id = game_cache.id;

    let scanned_element = ScannedGameElement::new(
        egs_id,
        game_cache.gamename.clone(),
        exe_path,
        lnk_path,
        _install_at,
    );

    let new_element_id = modules
        .collection_use_case()
        .create_collection_element(&scanned_element)
        .await?;

    modules
        .image_use_case()
        .save_icon_by_paths(
            &new_element_id,
            &scanned_element.exe_path,
            &scanned_element.lnk_path,
        )
        .await?;
    modules
        .image_use_case()
        .save_thumbnail(&new_element_id, &game_cache.thumbnail_url)
        .await?;
    Ok(modules
        .collection_use_case()
        .upsert_collection_element_thumbnail_size(&new_element_id)
        .await?)
}

