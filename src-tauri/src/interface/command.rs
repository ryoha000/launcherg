use std::sync::Arc;
use tauri::State;

use super::{
    error::CommandError,
    models::collection::{Collection, CollectionElement},
    module::{Modules, ModulesExt},
};
use crate::{
    domain::{
        collection::{NewCollectionElement, UpdateCollection},
        distance::get_comparable_distance,
        file::{get_icon_path, normalize, save_icon_to_png},
        Id,
    },
    infrastructure::repositoryimpl::migration::ONEPIECE_COLLECTION_ID,
    usecase::models::collection::CreateCollection,
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
) -> anyhow::Result<Vec<String>, CommandError> {
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
        return Ok(vec![]);
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

    Ok(new_elements.into_iter().map(|v| v.gamename).collect())
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

#[tauri::command]
pub async fn create_new_collection(
    modules: State<'_, Arc<Modules>>,
    name: String,
) -> anyhow::Result<Collection, CommandError> {
    Ok(modules
        .collection_use_case()
        .create_collection(CreateCollection::new(name))
        .await?
        .into())
}

#[tauri::command]
pub async fn update_collection(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    name: String,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_collection_by_id(UpdateCollection::new(Id::new(id), name))
        .await?)
}

#[tauri::command]
pub async fn delete_collection(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_collection_by_id(&Id::new(id))
        .await?)
}

#[tauri::command]
pub async fn upsert_collection_element(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    gamename: String,
    path: String,
) -> anyhow::Result<(), CommandError> {
    let new_element = NewCollectionElement::new(Id::new(id), gamename, path);
    modules
        .collection_use_case()
        .upsert_collection_element(&new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_icon(&new_element.path, &new_element.id)
        .await?;
    Ok(modules
        .collection_use_case()
        .add_collection_elements(&Id::new(ONEPIECE_COLLECTION_ID), &vec![Id::new(id)])
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_icon(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    path: String,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_collection_element_icon(&Id::new(id), path)
        .await?)
}

#[tauri::command]
pub async fn add_elements_to_collection(
    modules: State<'_, Arc<Modules>>,
    collection_id: i32,
    collection_element_ids: Vec<i32>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .add_collection_elements(
            &Id::new(collection_id),
            &collection_element_ids
                .into_iter()
                .map(|v| Id::new(v))
                .collect(),
        )
        .await?)
}

#[tauri::command]
pub async fn remove_elements_from_collection(
    modules: State<'_, Arc<Modules>>,
    collection_id: i32,
    collection_element_ids: Vec<i32>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .remove_collection_elements(
            &Id::new(collection_id),
            &collection_element_ids
                .into_iter()
                .map(|v| Id::new(v))
                .collect(),
        )
        .await?)
}

#[tauri::command]
pub async fn get_default_import_dirs() -> anyhow::Result<Vec<String>, CommandError> {
    let user_menu = dirs::home_dir()
        .ok_or(anyhow::anyhow!("cannot got home dir"))?
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .to_string_lossy()
        .to_string();

    let system_menu = "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs";

    Ok(vec![user_menu, system_menu.to_string()])
}

#[tauri::command]
pub async fn play_game(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
    is_run_as_admin: bool,
) -> anyhow::Result<(), CommandError> {
    let element = modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await?;
    Ok(modules
        .file_use_case()
        .start_game(element, is_run_as_admin)?)
}

#[tauri::command]
pub async fn get_play_time_minutes(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<f32, CommandError> {
    Ok(modules
        .file_use_case()
        .get_play_time_minutes(&Id::new(collection_element_id))?)
}

#[tauri::command]
pub async fn get_collection_element(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<CollectionElement, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await
        .and_then(|v| {
            Ok(CollectionElement {
                id: v.id.value,
                gamename: v.gamename,
                path: v.path,
                icon: get_icon_path(&v.id),
            })
        })?)
}

#[tauri::command]
pub async fn delete_collection_element(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .delete_collection_element_by_id(&Id::new(collection_element_id))
        .await?)
}
