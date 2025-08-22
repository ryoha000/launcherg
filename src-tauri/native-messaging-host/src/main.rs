mod models;

use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncReadExt, AsyncWriteExt};
use serde_json;
use serde::Deserialize;
use thiserror::Error;

use models::{
    common::{NativeMessageCase, NativeMessageTs, NativeResponseCase, NativeResponseTs, HealthCheckRequestTs, HealthCheckResultTs},
    sync::{DmmSyncGamesRequestTs, DlsiteSyncGamesRequestTs, SyncBatchResultTs},
    status::*,
    packs::{GetDmmPackIdsRequestTs, DmmPackIdsResponseTs},
};
use infrastructure::{
    repositoryimpl::{driver::Db as RepoDb, repository::Repositories},
    image_queue_worker::ImageQueueWorker,
};
use usecase::native_host_sync::{NativeHostSyncUseCase, DmmSyncGameParam, DlsiteSyncGameParam, EgsInfo};
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};
use domain::repository::{RepositoriesExt, dmm_pack::DmmPackRepository};

struct AppCtx {
    repositories: Arc<Repositories>,
    sync_usecase: NativeHostSyncUseCase<Repositories>,
    resolver: Arc<dyn SavePathResolver>,
}

#[derive(Debug)]
enum RequestFormat { Json }

type HostResult<T> = Result<T, HostError>;

#[derive(Debug, Error)]
enum HostError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Message too large: {0} bytes (limit 1048576)")]
    TooLarge(usize),
}

fn anyhow_chain_to_string(err: &anyhow::Error) -> String {
    err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(": ")
}

async fn read_framed() -> HostResult<Option<Vec<u8>>> {
    let mut stdin = tokio_io::stdin();

    let mut length_bytes = [0u8; 4];
    if let Err(e) = stdin.read_exact(&mut length_bytes).await {
        if e.kind() == ErrorKind::UnexpectedEof {
            return Ok(None);
        }
        return Err(HostError::Io(e));
    }

    let length = u32::from_le_bytes(length_bytes) as usize;
    if length > 1024 * 1024 {
        return Err(HostError::TooLarge(length));
    }

    let mut message_bytes = vec![0u8; length];
    stdin.read_exact(&mut message_bytes).await?;
    Ok(Some(message_bytes))
}

async fn send_error_response(request_id: &str, message: String) -> Result<(), Box<dyn std::error::Error>> {
    let response = NativeResponseTs { success: false, error: message, request_id: request_id.to_string(), response: None };
    send_response_json(&response).await
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    let db_path = usecase::native_host_sync::db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repositories = Arc::new(Repositories::new(repo_db));
    let resolver = Arc::new(DirsSavePathResolver::default());
    let sync_usecase = NativeHostSyncUseCase::new(repositories.clone(), resolver.clone());
    let ctx = AppCtx { repositories, sync_usecase, resolver };

    log::info!("Native Messaging Host started");

    loop {
        match handle_message(&ctx).await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(e) => {
                log::error!("Error handling message: {}", e);
                break;
            }
        }
    }

    log::info!("Native Messaging Host stopped");
}

async fn handle_message(ctx: &AppCtx) -> Result<bool, Box<dyn std::error::Error>> {
    let message_bytes = match read_framed().await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(false),
        Err(HostError::TooLarge(length)) => {
            let error_msg = format!("Message too large: {} bytes (limit 1048576)", length);
            send_error_response("", error_msg).await?;
            return Ok(true);
        }
        Err(e) => return Err(e.into()),
    };

    let (message, format) = match parse_message(&message_bytes) {
        Ok(v) => v,
        Err(err) => {
            let mut request_id = String::new();
            if let Ok(s) = std::str::from_utf8(&message_bytes) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                    if let Some(id) = v.get("request_id").and_then(|x| x.as_str()) {
                        request_id = id.to_string();
                    }
                }
            }

            send_error_response(&request_id, err.to_string()).await?;
            return Ok(true);
        }
    };

    let response = match &message.message {
        NativeMessageCase::SyncDmmGames(req) => handle_sync_dmm_games(ctx, req, &message.request_id).await,
        NativeMessageCase::SyncDlsiteGames(req) => handle_sync_dlsite_games(ctx, req, &message.request_id).await,
        NativeMessageCase::GetDmmPackIds(req) => handle_get_dmm_pack_ids(ctx, req, &message.request_id).await,
        NativeMessageCase::GetStatus(_) => NativeResponseTs { success: false, error: "GetStatus is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::SetConfig(_) => NativeResponseTs { success: false, error: "SetConfig is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::HealthCheck(_) => handle_health_check(&HealthCheckRequestTs {}, &message.request_id),
    };

    let _ = format; // currently always Json
    send_response_json(&response).await?;

    // 画像キューの drain は同期時のみ
    match &message.message {
        NativeMessageCase::SyncDmmGames(_) | NativeMessageCase::SyncDlsiteGames(_) => {
            let worker = ImageQueueWorker::new(ctx.repositories.clone(), ctx.resolver.clone());
            let _ = worker.drain_until_empty().await;
            return Ok(false);
        }
        _ => {}
    }

    Ok(true)
}

async fn handle_sync_dmm_games(ctx: &AppCtx, request: &DmmSyncGamesRequestTs, request_id: &str) -> NativeResponseTs {
    let input_ids: Vec<String> = request.games.iter().map(|g| g.id.clone()).collect();
    let params: Vec<DmmSyncGameParam> = request
        .games
        .iter()
        .map(|g| DmmSyncGameParam {
            store_id: g.id.clone(),
            category: g.category.clone(),
            subcategory: g.subcategory.clone(),
            gamename: g.title.clone(),
            image_url: g.image_url.clone(),
            egs: g.egs_info.as_ref().map(|e| EgsInfo {
                erogamescape_id: e.erogamescape_id,
                gamename: e.gamename.clone(),
                gamename_ruby: e.gamename_ruby.clone(),
                brandname: e.brandname.clone(),
                brandname_ruby: e.brandname_ruby.clone(),
                sellday: e.sellday.clone(),
                is_nukige: e.is_nukige,
            }),
        })
        .collect();
    match ctx.sync_usecase.sync_dmm_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponseTs { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(NativeResponseCase::SyncGamesResult(result)) }
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            let result = SyncBatchResultTs {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            NativeResponseTs { success: false, error: err_msg, request_id: request_id.to_string(), response: Some(NativeResponseCase::SyncGamesResult(result)) }
        }
    }
}

async fn handle_sync_dlsite_games(ctx: &AppCtx, request: &DlsiteSyncGamesRequestTs, request_id: &str) -> NativeResponseTs {
    let input_ids: Vec<String> = request.games.iter().map(|g| g.id.clone()).collect();
    let params: Vec<DlsiteSyncGameParam> = request
        .games
        .iter()
        .map(|g| DlsiteSyncGameParam {
            store_id: g.id.clone(),
            category: g.category.clone(),
            gamename: g.title.clone(),
            image_url: g.image_url.clone(),
            egs: g.egs_info.as_ref().map(|e| EgsInfo {
                erogamescape_id: e.erogamescape_id,
                gamename: e.gamename.clone(),
                gamename_ruby: e.gamename_ruby.clone(),
                brandname: e.brandname.clone(),
                brandname_ruby: e.brandname_ruby.clone(),
                sellday: e.sellday.clone(),
                is_nukige: e.is_nukige,
            }),
        })
        .collect();
    match ctx.sync_usecase.sync_dlsite_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponseTs { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(NativeResponseCase::SyncGamesResult(result)) }
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            let result = SyncBatchResultTs {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            NativeResponseTs { success: false, error: err_msg, request_id: request_id.to_string(), response: Some(NativeResponseCase::SyncGamesResult(result)) }
        }
    }
}

async fn handle_get_dmm_pack_ids(ctx: &AppCtx, _request: &GetDmmPackIdsRequestTs, request_id: &str) -> NativeResponseTs {
    let list = match ctx.repositories.dmm_pack_repository().list().await {
        Ok(v) => v,
        Err(e) => {
            return NativeResponseTs { success: false, error: anyhow_chain_to_string(&e), request_id: request_id.to_string(), response: None }
        }
    };
    let store_ids: Vec<String> = list.into_iter().map(|m| m.store_id).collect();
    let result = DmmPackIdsResponseTs { store_ids };
    NativeResponseTs { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(NativeResponseCase::DmmPackIds(result)) }
}

fn handle_health_check(_request: &HealthCheckRequestTs, request_id: &str) -> NativeResponseTs {
    let result = HealthCheckResultTs {
        message: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    NativeResponseTs { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(NativeResponseCase::HealthCheckResult(result)) }
}

fn parse_message(message_bytes: &[u8]) -> HostResult<(NativeMessageTs, RequestFormat)> {
    let json_str = std::str::from_utf8(message_bytes)?;
    let message: NativeMessageTs = serde_json::from_str(json_str)?;
    Ok((message, RequestFormat::Json))
}

async fn send_response_json(response: &NativeResponseTs) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = tokio_io::stdout();

    let json_response = serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize JSON response: {}", e))?;
    let json_bytes = json_response.as_bytes();
    let length = json_bytes.len() as u32;
    stdout.write_all(&length.to_le_bytes()).await?;
    stdout.write_all(json_bytes).await?;
    stdout.flush().await?;
    Ok(())
}

