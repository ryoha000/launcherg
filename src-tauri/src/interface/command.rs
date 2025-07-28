use std::sync::Arc;
use tauri::AppHandle;
use tauri::State;

use super::models::all_game_cache::AllGameCacheOne;
use super::{
    error::CommandError,
    models::collection::CollectionElement,
    module::{Modules, ModulesExt},
};
use crate::domain::file::get_lnk_metadatas;
use crate::domain::windows::proctail::{
    HealthCheckResult, ProcTailEvent, ServiceStatus, WatchTarget,
};
use crate::infrastructure::windowsimpl::proctail_manager::{
    ProcTailManagerStatus, ProcTailVersion,
};
use crate::{
    domain::{
        collection::{NewCollectionElement, ScannedGameElement},
        distance::get_comparable_distance,
        file::{get_file_created_at_sync, normalize},
        pubsub::{ProgressLivePayload, ProgressPayload, PubSubService},
        Id,
    },
    usecase::models::collection::CreateCollectionElementDetail,
};

#[tauri::command]
pub async fn create_elements_in_pc(
    modules: State<'_, Arc<Modules>>,
    handle: AppHandle,
    explore_dir_paths: Vec<String>,
    use_cache: bool,
) -> anyhow::Result<Vec<String>, CommandError> {
    let handle = Arc::new(handle);
    let pubsub = modules.pubsub();

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

    if let Err(e) = pubsub.notify(
        "progress",
        ProgressPayload::new(format!(
            "指定したフォルダの .lnk .exe ファイルを取得しました。ファイル数: {}",
            explore_files.len()
        )),
    ) {
        return Err(CommandError::Anyhow(anyhow::anyhow!(e.to_string())));
    }
    if let Err(e) = pubsub.notify(
        "progresslive",
        ProgressLivePayload::new(Some(explore_files.len() as i32)),
    ) {
        return Err(CommandError::Anyhow(anyhow::anyhow!(e.to_string())));
    }

    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_all_game_cache()
        .await?;

    let new_elements_with_data = modules
        .file_use_case()
        .filter_files_to_collection_elements(
            &handle,
            explore_files.clone(),
            all_game_cache,
            Arc::new(pubsub.clone()),
        )
        .await?;

    let new_elements_game_caches = modules
        .all_game_cache_use_case()
        .get_by_ids(new_elements_with_data.iter().map(|v| v.id.value).collect())
        .await?;

    // ゲーム名を保存しておく（戻り値で使用）
    let gamenames: Vec<String> = new_elements_game_caches
        .iter()
        .map(|v| v.gamename.clone())
        .collect();

    modules
        .collection_use_case()
        .concurency_save_thumbnails(
            &handle,
            new_elements_game_caches
                .into_iter()
                .map(|v| (Id::new(v.id), v.thumbnail_url))
                .collect(),
        )
        .await?;

    modules
        .collection_use_case()
        .upsert_collection_elements(&new_elements_with_data)
        .await?;

    let new_element_ids = new_elements_with_data
        .iter()
        .map(|v| v.id.clone())
        .collect::<Vec<Id<_>>>();
    modules
        .collection_use_case()
        .concurency_upsert_collection_element_thumbnail_size(&handle, new_element_ids)
        .await?;

    modules
        .explored_cache_use_case()
        .add_cache(explore_files)
        .await?;

    Ok(gamenames)
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
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    base64_image: String,
) -> anyhow::Result<String, CommandError> {
    Ok(modules
        .file_use_case()
        .upload_image(&Arc::new(handle), id, base64_image)
        .await?)
}

#[tauri::command]
pub async fn upsert_collection_element(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    exe_path: Option<String>,
    lnk_path: Option<String>,
    game_cache: AllGameCacheOne,
) -> anyhow::Result<(), CommandError> {
    let _install_at;
    if let Some(path) = exe_path.clone() {
        _install_at = get_file_created_at_sync(&path);
    } else if let Some(path) = lnk_path.clone() {
        let metadatas = get_lnk_metadatas(vec![path.as_str()])?;
        let metadata = metadatas
            .get(path.as_str())
            .ok_or(anyhow::anyhow!("metadata cannot get"))?;
        println!(
            "metadata.path: {}, metadata.icon: {}",
            metadata.path, metadata.icon
        );
        _install_at = get_file_created_at_sync(&metadata.path);
    } else {
        _install_at = None;
    }
    let element_id = Id::new(game_cache.id);
    let handle = Arc::new(handle);

    // ScannedGameElementを作成
    let scanned_element = ScannedGameElement::new(
        element_id.clone(),
        exe_path,
        lnk_path,
        _install_at,
    );

    // 関連データを含むコレクション要素を作成
    modules
        .collection_use_case()
        .create_collection_element(&scanned_element)
        .await?;

    // アイコンを保存
    let new_element = NewCollectionElement::new(element_id.clone());
    modules
        .collection_use_case()
        .save_element_icon(&handle, &new_element)
        .await?;
    modules
        .collection_use_case()
        .save_element_thumbnail(&handle, &new_element.id, game_cache.thumbnail_url)
        .await?;
    Ok(modules
        .collection_use_case()
        .upsert_collection_element_thumbnail_size(&handle, &new_element.id)
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_thumbnails(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    ids: Vec<i32>,
) -> anyhow::Result<(), CommandError> {
    let all_game_cache = modules
        .all_game_cache_use_case()
        .get_by_ids(ids.clone())
        .await?;
    let handle = Arc::new(handle);
    modules
        .collection_use_case()
        .concurency_save_thumbnails(
            &handle,
            all_game_cache
                .into_iter()
                .map(|v| (Id::new(v.id), v.thumbnail_url))
                .collect(),
        )
        .await?;
    Ok(modules
        .collection_use_case()
        .concurency_upsert_collection_element_thumbnail_size(
            &handle,
            ids.into_iter().map(|v| Id::new(v)).collect(),
        )
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_icon(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    path: String,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .update_collection_element_icon(&Arc::new(handle), &Id::new(id), path)
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
) -> anyhow::Result<Option<u32>, CommandError> {
    let element = modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await?;
    let process_id = modules
        .file_use_case()
        .start_game(element, is_run_as_admin)?;
    modules
        .collection_use_case()
        .update_element_last_play_at(&Id::new(collection_element_id))
        .await?;
    Ok(process_id)
}

#[tauri::command]
pub async fn get_play_time_minutes(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<f32, CommandError> {
    Ok(modules
        .file_use_case()
        .get_play_time_minutes(&Arc::new(handle), &Id::new(collection_element_id))?)
}

#[tauri::command]
pub async fn get_collection_element(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<CollectionElement, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_element_by_element_id(&Id::new(collection_element_id))
        .await
        .and_then(|v| Ok(CollectionElement::from_domain(&Arc::new(handle), v)))?)
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
    for detail in details {
        let info = crate::domain::collection::NewCollectionElementInfo::new(
            Id::new(detail.collection_element_id),
            detail.gamename,
            detail.gamename_ruby,
            detail.brandname,
            detail.brandname_ruby,
            detail.sellday,
            detail.is_nukige,
        );
        modules
            .collection_use_case()
            .upsert_collection_element_info(&info)
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_all_elements(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<CollectionElement>, CommandError> {
    let handle = &Arc::new(handle);
    Ok(modules
        .collection_use_case()
        .get_all_elements(&handle)
        .await?
        .into_iter()
        .map(|v| CollectionElement::from_domain(&handle, v))
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

#[tauri::command]
pub async fn save_screenshot_by_pid(
    handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
    process_id: u32,
) -> anyhow::Result<String, CommandError> {
    let upload_path = modules
        .file_use_case()
        .get_new_upload_image_path(&Arc::new(handle), work_id)?;
    modules
        .process_use_case()
        .save_screenshot_by_pid(process_id, &upload_path)
        .await?;
    Ok(upload_path)
}

// ProcTail commands
#[derive(serde::Deserialize)]
pub struct AddWatchTargetRequest {
    #[serde(rename = "processId")]
    pub process_id: u32,
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_add_watch_target(
    modules: State<'_, Arc<Modules>>,
    request: AddWatchTargetRequest,
) -> anyhow::Result<WatchTarget, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_add_watch_target(request.process_id, &request.tag)
        .await?)
}

#[derive(serde::Deserialize)]
pub struct RemoveWatchTargetRequest {
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_remove_watch_target(
    modules: State<'_, Arc<Modules>>,
    request: RemoveWatchTargetRequest,
) -> anyhow::Result<u32, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_remove_watch_target(&request.tag)
        .await?)
}

#[tauri::command]
pub async fn proctail_get_watch_targets(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<WatchTarget>, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_get_watch_targets()
        .await?)
}

#[derive(serde::Deserialize)]
pub struct GetEventsRequest {
    pub tag: String,
    pub count: Option<u32>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
}

#[tauri::command]
pub async fn proctail_get_recorded_events(
    modules: State<'_, Arc<Modules>>,
    request: GetEventsRequest,
) -> anyhow::Result<Vec<ProcTailEvent>, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_get_recorded_events(&request.tag, request.count, request.event_type.as_deref())
        .await?)
}

#[derive(serde::Deserialize)]
pub struct ClearEventsRequest {
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_clear_events(
    modules: State<'_, Arc<Modules>>,
    request: ClearEventsRequest,
) -> anyhow::Result<u32, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_clear_events(&request.tag)
        .await?)
}

#[tauri::command]
pub async fn proctail_get_status(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ServiceStatus, CommandError> {
    Ok(modules.process_use_case().proctail_get_status().await?)
}

#[tauri::command]
pub async fn proctail_health_check(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<HealthCheckResult, CommandError> {
    Ok(modules.process_use_case().proctail_health_check().await?)
}

#[tauri::command]
pub async fn proctail_is_service_available(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_is_service_available()
        .await?)
}

// ProcTail Manager commands
#[tauri::command]
pub async fn proctail_manager_get_status(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ProcTailManagerStatus, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_get_status()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_get_latest_version(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ProcTailVersion, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_get_latest_version()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_is_update_available(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_is_update_available()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_download_and_install(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_download_and_install()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_start(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.process_use_case().proctail_manager_start().await?)
}

#[tauri::command]
pub async fn proctail_manager_stop(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.process_use_case().proctail_manager_stop().await?)
}

#[tauri::command]
pub async fn proctail_manager_is_running(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_is_running()
        .await?)
}
