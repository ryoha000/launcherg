use std::sync::Arc;
use std::sync::Mutex;
use tauri::State;
use tauri::Window;

use super::models::all_game_cache::AllGameCacheOne;
use super::models::collection::ProgressLivePayload;
use super::{
    error::CommandError,
    models::collection::CollectionElement,
    module::{Modules, ModulesExt},
};
use crate::domain::file::get_lnk_metadatas;
use crate::interface::models::collection::ProgressPayload;
use crate::{
    domain::{
        collection::NewCollectionElement,
        distance::get_comparable_distance,
        file::{get_file_created_at_sync, normalize},
        Id,
    },
    infrastructure::repositoryimpl::migration::ONEPIECE_COLLECTION_ID,
    usecase::models::collection::CreateCollectionElementDetail,
};

#[tauri::command]
pub async fn create_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    window: Window,
    explore_dir_paths: Vec<String>,
    use_cache: bool,
) -> anyhow::Result<Vec<String>, CommandError> {
    let window = Arc::new(window);
    let emit_progress = Arc::new(|message| {
        if let Err(e) = window.emit("progress", ProgressPayload::new(message)) {
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    });
    let cloned_window = Arc::clone(&window);
    let process_each_game_file_callback = Arc::new(Mutex::new(move || {
        if let Err(e) = cloned_window.emit("progresslive", ProgressLivePayload::new(None)) {
            return Err(anyhow::anyhow!(e.to_string()));
        }
        Ok(())
    }));

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

    emit_progress(format!(
        "指定したフォルダの .lnk .exe ファイルを取得しました。ファイル数: {}",
        explore_files.len()
    ))?;
    if let Err(e) = window.emit(
        "progresslive",
        ProgressLivePayload::new(Some(explore_files.len() as i32)),
    ) {
        return Err(CommandError::Anyhow(anyhow::anyhow!(e.to_string())));
    }

    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_all_game_cache()
        .await?;

    let new_elements = modules
        .file_use_case()
        .filter_files_to_collection_elements(
            explore_files.clone(),
            all_game_cache,
            emit_progress,
            process_each_game_file_callback,
        )
        .await?;

    let new_elements_game_caches = modules
        .all_game_cache_use_case()
        .get_by_ids(new_elements.iter().map(|v| v.id.value).collect())
        .await?;
    modules
        .collection_use_case()
        .concurency_save_thumbnails(
            new_elements_game_caches
                .into_iter()
                .map(|v| (Id::new(v.id), v.thumbnail_url))
                .collect(),
        )
        .await?;

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements)
        .await?;

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
pub async fn upsert_collection_element(
    modules: State<'_, Arc<Modules>>,
    exe_path: Option<String>,
    lnk_path: Option<String>,
    game_cache: AllGameCacheOne,
) -> anyhow::Result<(), CommandError> {
    let install_at;
    if let Some(path) = exe_path.clone() {
        install_at = get_file_created_at_sync(&path);
    } else if let Some(path) = lnk_path.clone() {
        install_at = get_file_created_at_sync(&path);
    } else {
        install_at = None;
    }
    let new_element = NewCollectionElement::new(
        Id::new(game_cache.id),
        game_cache.gamename,
        exe_path,
        lnk_path,
        install_at,
    );
    modules
        .collection_use_case()
        .upsert_collection_element(&new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_icon(&new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_thumbnail(&new_element.id, game_cache.thumbnail_url)
        .await?;
    Ok(modules
        .collection_use_case()
        .add_collection_elements(
            &Id::new(ONEPIECE_COLLECTION_ID),
            &vec![Id::new(game_cache.id)],
        )
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
    modules
        .file_use_case()
        .start_game(element, is_run_as_admin)?;
    Ok(modules
        .collection_use_case()
        .update_element_last_play_at(&Id::new(collection_element_id))
        .await?)
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
        .and_then(|v| Ok(v.into()))?)
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

#[tauri::command]
pub async fn get_not_registered_detail_element_ids(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<i32>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_not_registered_detail_element_ids()
        .await?
        .into_iter()
        .map(|v| v.value)
        .collect())
}

#[tauri::command]
pub async fn create_element_details(
    modules: State<'_, Arc<Modules>>,
    details: Vec<CreateCollectionElementDetail>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .create_element_details(details.into_iter().map(|v| v.into()).collect())
        .await?)
}

#[tauri::command]
pub async fn get_all_elements(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<CollectionElement>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_all_elements()
        .await?
        .into_iter()
        .map(|v| v.into())
        .collect())
}

#[tauri::command]
pub async fn update_element_like(
    modules: State<'_, Arc<Modules>>,
    id: i32,
    is_like: bool,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_element_like_at(&Id::new(id), is_like)
        .await?)
}

#[tauri::command]
pub fn open_folder(path: String) -> anyhow::Result<(), CommandError> {
    let p = std::path::Path::new(&path);
    let path = match p.is_file() {
        true => p
            .parent()
            .ok_or(anyhow::anyhow!("parent not found"))?
            .to_string_lossy()
            .to_string(),
        false => path,
    };
    let err_msg = anyhow::anyhow!("Failed to open folder at path: {}", path);
    std::process::Command::new("explorer")
        .arg(path)
        .output()
        .map_err(|_| err_msg)?;

    Ok(())
}

#[tauri::command]
pub async fn get_all_game_cache_last_updated(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(i32, String), CommandError> {
    let last_updated = modules
        .all_game_cache_use_case()
        .get_cache_last_updated()
        .await?;
    Ok((last_updated.0, last_updated.1.to_rfc3339()))
}

#[tauri::command]
pub async fn update_all_game_cache(
    modules: State<'_, Arc<Modules>>,
    game_caches: Vec<AllGameCacheOne>,
) -> anyhow::Result<(), CommandError> {
    modules
        .all_game_cache_use_case()
        .update_all_game_cache(game_caches.into_iter().map(|v| v.into()).collect())
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_game_candidates(
    modules: State<'_, Arc<Modules>>,
    filepath: String,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_all_game_cache()
        .await?;

    Ok(modules
        .file_use_case()
        .get_game_candidates(all_game_cache, filepath)
        .await?
        .into_iter()
        .map(|c| (c.id, c.gamename))
        .collect())
}

#[tauri::command]
pub async fn get_exe_path_by_lnk(filepath: String) -> anyhow::Result<String, CommandError> {
    if !filepath.to_lowercase().ends_with("lnk") {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "filepath is not ends with lnk"
        )));
    }

    let p: &str = &filepath;
    let metadatas = get_lnk_metadatas(vec![p])?;
    if let Some(meta) = metadatas.get(p) {
        return Ok(meta.path.clone());
    } else {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "cannot get lnk metadata"
        )));
    }
}

#[tauri::command]
pub async fn get_game_cache_by_id(
    modules: State<'_, Arc<Modules>>,
    id: i32,
) -> anyhow::Result<Option<AllGameCacheOne>, CommandError> {
    Ok(modules
        .all_game_cache_use_case()
        .get(id)
        .await?
        .and_then(|v| Some(v.into())))
}
