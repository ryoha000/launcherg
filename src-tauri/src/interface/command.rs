use std::sync::Arc;
use tauri::State;

use super::{
    error::CommandError,
    models::collection::{Collection, CollectionElement},
    module::{Modules, ModulesExt},
};
use crate::{
    domain::{
        distance::get_comparable_distance,
        file::{get_icon_path, normalize},
        Id,
    },
    infrastructure::repositoryimpl::migration::ONEPIECE_COLLECTION_ID,
};

#[tauri::command]
pub async fn get_all_collections(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<Collection>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_all_collections()
        .await?
        .into_iter()
        .map(|v| Collection {
            id: v.id.value,
            name: v.name,
            created_at: v.created_at,
            updated_at: v.updated_at,
        })
        .collect())
}

#[tauri::command]
pub async fn get_collection_elements(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> anyhow::Result<Vec<CollectionElement>, CommandError> {
    let id = Id::new(id);
    Ok(modules
        .collection_use_case()
        .get_elements_by_id(&id)
        .await?
        .into_iter()
        .map(|v| CollectionElement {
            id: v.id.value,
            gamename: v.gamename,
            path: v.path,
            icon: get_icon_path(&v.id),
        })
        .collect())
}

#[tauri::command]
pub async fn add_collection_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    explore_dir_paths: Vec<String>,
    use_cache: bool,
    adding_collection_id: Option<i32>,
) -> anyhow::Result<(), CommandError> {
    let explored_caches = modules.explored_cache_use_case().get_cache().await?;
    let explore_files: Vec<String> = modules
        .file_use_case()
        .concurency_get_file_paths(explore_dir_paths)
        .await?
        .into_iter()
        .filter_map(|v| match use_cache && explored_caches.contains(&v) {
            true => None,
            false => Some(v),
        })
        .collect();

    if explore_files.len() == 0 {
        println!("all file is match to cache");
        return Ok(());
    }

    // TODO: message end get lnk,exe

    let new_elements = modules
        .file_use_case()
        .filter_files_to_collection_elements(explore_files.clone())
        .await?;

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

    modules
        .explored_cache_use_case()
        .add_cache(explore_files)
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_nearest_key_and_distance(
    key: String,
    calculate_distance_kv: Vec<(String, String)>,
) -> anyhow::Result<(String, f32), CommandError> {
    let key = normalize(&key);
    let normalized_kv = calculate_distance_kv
        .into_iter()
        .map(|v| (normalize(&v.0), normalize(&v.1)))
        .collect::<Vec<(String, String)>>();

    for (comp_key, comp_value) in normalized_kv.iter() {
        if key == *comp_key {
            return Ok((comp_value.to_string(), 1.0));
        }
    }

    let mut max_distance = 0.0;
    let mut max_distance_value = None;
    for (comp_key, comp_value) in normalized_kv.into_iter() {
        let distance = get_comparable_distance(&key, &comp_key);
        if max_distance < distance {
            max_distance = distance;
            max_distance_value = Some(comp_value);
        }
    }

    match max_distance_value {
        Some(value) => Ok((value, max_distance)),
        _ => Err(CommandError::Anyhow(anyhow::anyhow!(
            "maybe calculate_distance_kv is empty."
        ))),
    }
}

#[tauri::command]
pub async fn upload_image(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    base64_image: String,
) -> anyhow::Result<String, CommandError> {
    Ok(modules
        .file_use_case()
        .upload_image(id, base64_image)
        .await?)
}

#[tauri::command]
pub async fn get_memo_path(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> anyhow::Result<String, CommandError> {
    Ok(modules.file_use_case().get_memo_path(id).await?)
}
