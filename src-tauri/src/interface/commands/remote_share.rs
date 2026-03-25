use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::remote_share::RemoteShareSettingsVm;
use crate::interface::models::work_details::WorkDetailsVm;
use crate::interface::module::{Modules, ModulesExt};
use futures::stream::{self, StreamExt, TryStreamExt};

use usecase::remote_share::{
    CommitSyncResponse,
    RemoteShareUploadedImage,
    RemoteShareWorkInput,
};

#[tauri::command]
pub async fn get_remote_share_settings(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<RemoteShareSettingsVm, CommandError> {
    Ok(modules
        .app_settings_use_case()
        .get_remote_share_settings()
        .await?
        .into())
}

#[tauri::command]
pub async fn set_remote_share_settings(
    modules: State<'_, Arc<Modules>>,
    settings: RemoteShareSettingsVm,
) -> anyhow::Result<RemoteShareSettingsVm, CommandError> {
    Ok(modules
        .app_settings_use_case()
        .set_remote_share_settings(settings.into())
        .await?
        .into())
}

#[tauri::command]
pub async fn register_remote_share_device(
    modules: State<'_, Arc<Modules>>,
    settings: RemoteShareSettingsVm,
) -> anyhow::Result<RemoteShareSettingsVm, CommandError> {
    let server_base_url = settings
        .server_base_url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("serverBaseUrl is required"))?;
    let device_secret = settings
        .device_secret
        .clone()
        .ok_or_else(|| anyhow::anyhow!("deviceSecret is required"))?;

    let remote_share_use_case = crate::usecase::remote_share::RemoteShareUseCase::new();
    let device_id = remote_share_use_case
        .register_device(&server_base_url, &device_secret)
        .await?;

    let saved = modules
        .app_settings_use_case()
        .set_remote_share_settings(RemoteShareSettingsVm {
            device_secret: Some(device_secret),
            device_id: Some(device_id),
            server_base_url: Some(server_base_url),
            last_remote_sync_at: settings.last_remote_sync_at,
        }
        .into())
        .await?;

    Ok(saved.into())
}

#[tauri::command]
pub async fn sync_remote_share_works(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<RemoteShareSettingsVm, CommandError> {
    let current = modules.app_settings_use_case().get_remote_share_settings().await?;
    let server_base_url = current
        .remote_share_server_base_url
        .clone()
        .ok_or_else(|| anyhow::anyhow!("serverBaseUrl is required"))?;
    let device_secret = current
        .remote_share_device_secret
        .clone()
        .ok_or_else(|| anyhow::anyhow!("deviceSecret is required"))?;
    let device_id = current
        .remote_share_device_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("deviceId is required"))?;

    let rows = modules.work_use_case().list_all_details().await?;
    let resolver = modules.save_path_resolver().clone();
    let remote_share_use_case = crate::usecase::remote_share::RemoteShareUseCase::new();

    let candidates = rows
        .into_iter()
        .map(|row| WorkDetailsVm::from_work_details_with_resolver(row, resolver.as_ref()))
        .map(|work| to_remote_share_work(work))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let works = candidates.iter().map(|candidate| candidate.work.clone()).collect::<Vec<_>>();
    let thumbnail_paths: HashMap<String, PathBuf> = candidates
        .iter()
        .filter_map(|candidate| {
            candidate.thumbnail_path.as_ref().map(|path| {
                (candidate.work.dedupe_key(), path.clone())
            })
        })
        .fold(HashMap::new(), |mut acc, (dedupe_key, path)| {
            acc.entry(dedupe_key).or_insert(path);
            acc
        });

    let upload_targets = remote_share_use_case
        .prepare_sync_works(
            &server_base_url,
            &device_id,
            &device_secret,
            works.clone(),
        )
        .await?;

    let upload_jobs = upload_targets
        .into_iter()
        .filter_map(|target| {
            thumbnail_paths
                .get(&target.dedupe_key)
                .map(|path| (target, path.clone()))
        })
        .collect::<Vec<_>>();

    let uploaded_images = stream::iter(upload_jobs)
        .map(|(target, path)| {
            let remote_share_use_case = remote_share_use_case.clone();
            async move {
                remote_share_use_case
                    .upload_thumbnail(&target.upload_url, &target.content_type, &path)
                    .await?;
                Ok::<RemoteShareUploadedImage, anyhow::Error>(RemoteShareUploadedImage {
                    dedupe_key: target.dedupe_key,
                    image_key: target.image_key,
                })
            }
        })
        .buffer_unordered(4)
        .try_collect::<Vec<_>>()
        .await?;

    let response: CommitSyncResponse = remote_share_use_case
        .commit_sync_works(
            &server_base_url,
            &device_id,
            &device_secret,
            works,
            uploaded_images,
        )
        .await?;

    let saved = modules
        .app_settings_use_case()
        .set_remote_share_settings(RemoteShareSettingsVm {
            device_secret: Some(device_secret),
            device_id: Some(device_id),
            server_base_url: Some(server_base_url),
            last_remote_sync_at: Some(response.last_synced_at),
        }
        .into())
        .await?;

    Ok(saved.into())
}

#[tauri::command]
pub async fn get_remote_share_url(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<String, CommandError> {
    let current = modules.app_settings_use_case().get_remote_share_settings().await?;
    let server_base_url = current
        .remote_share_server_base_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("serverBaseUrl is required"))?;
    let device_id = current
        .remote_share_device_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("deviceId is required"))?;

    Ok(crate::usecase::remote_share::RemoteShareUseCase::new()
        .build_share_url(server_base_url, device_id)?)
}

struct RemoteShareWorkCandidate {
    work: RemoteShareWorkInput,
    thumbnail_path: Option<PathBuf>,
}

fn to_remote_share_work(work: WorkDetailsVm) -> anyhow::Result<RemoteShareWorkCandidate> {
    let thumbnail = match work.thumbnail {
        Some(thumbnail) => {
            let thumbnail_path = thumbnail.path.clone();
            let path = Path::new(&thumbnail_path);
            if path.exists() {
                Some((thumbnail_path.clone(), infer_content_type(&thumbnail_path).to_string(), thumbnail.width, thumbnail.height))
            } else {
                None
            }
        }
        None => None,
    };

    let (thumbnail_path, thumbnail) = match thumbnail {
        Some((path, content_type, width, height)) => (
            Some(PathBuf::from(path)),
            Some(crate::usecase::remote_share::RemoteShareThumbnailInput {
                content_type: content_type.to_string(),
                width,
                height,
            }),
        ),
        None => (None, None),
    };

    Ok(RemoteShareWorkCandidate {
        work: RemoteShareWorkInput {
            work_id: work.id,
            title: work.title,
            erogamescape_id: work.erogamescape_id,
            thumbnail,
        },
        thumbnail_path,
    })
}

fn infer_content_type(path: &str) -> &str {
    if path.to_ascii_lowercase().ends_with(".webp") {
        "image/webp"
    } else if path.to_ascii_lowercase().ends_with(".jpg")
        || path.to_ascii_lowercase().ends_with(".jpeg")
    {
        "image/jpeg"
    } else {
        "image/png"
    }
}
