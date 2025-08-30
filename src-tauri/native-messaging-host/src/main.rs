mod models;

use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncReadExt, AsyncWriteExt};
use serde_json;
use thiserror::Error;

use models::{
    common::{NativeMessageCase, NativeMessageTs, NativeResponseCase, NativeResponseTs, HealthCheckRequestTs, HealthCheckResultTs},
    sync::{DmmSyncGamesRequestTs, DlsiteSyncGamesRequestTs, SyncBatchResultTs},
    packs::{GetDmmOmitWorksRequestTs, DmmOmitWorkItemTs, DmmOmitDmmPartTs},
};
use infrastructure::{
    sqliterepository::{driver::Db as RepoDb, sqliterepository::{SqliteRepository, RepositoryExecutor}},
    image_queue_worker::ImageQueueWorker,
};
use usecase::native_host_sync::{NativeHostSyncUseCase, DmmSyncGameParam, DlsiteSyncGameParam, EgsInfo};
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

struct AppCtx {
    repositories: Arc<tokio::sync::Mutex<SqliteRepository<'static>>>,
    sync_usecase: NativeHostSyncUseCase<SqliteRepository<'static>>,
    resolver: Arc<dyn SavePathResolver>,
}

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

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    let db_path = usecase::native_host_sync::db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repo = SqliteRepository::new(RepositoryExecutor::OwnedPool(repo_db.pool_arc()));
    let repositories = Arc::new(tokio::sync::Mutex::new(repo));
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

async fn handle_message(ctx: &AppCtx) -> HostResult<bool> {
    let message_bytes = match read_framed().await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(false),
        Err(HostError::TooLarge(length)) => {
            let error_msg = HostError::TooLarge(length).to_string();
            send_error_response("", error_msg).await?;
            return Ok(true);
        }
        Err(e) => return Err(e),
    };

    let message = match parse_message(&message_bytes) {
        Ok(v) => v,
        Err(err) => {
            let mut request_id = String::new();
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&message_bytes) {
                if let Some(id) = v.get("request_id").and_then(|x| x.as_str()) {
                    request_id = id.to_string();
                }
            }

            send_error_response(&request_id, err.to_string()).await?;
            return Ok(true);
        }
    };

    let response = match &message.message {
        NativeMessageCase::SyncDmmGames(req) => handle_sync_dmm_games(ctx, req, &message.request_id).await,
        NativeMessageCase::SyncDlsiteGames(req) => handle_sync_dlsite_games(ctx, req, &message.request_id).await,
        NativeMessageCase::GetDmmOmitWorks(req) => handle_get_dmm_omit_works(ctx, req, &message.request_id).await,
        NativeMessageCase::GetStatus(_) => NativeResponseTs { success: false, error: "GetStatus is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::SetConfig(_) => NativeResponseTs { success: false, error: "SetConfig is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::HealthCheck(_) => handle_health_check(&HealthCheckRequestTs {}, &message.request_id),
    };

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
    let (input_ids, params) = to_dmm_params(request);
    match ctx.sync_usecase.sync_dmm_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs { success_count, error_count: 0, errors: vec![], synced_games: input_ids.clone() };
            ok(request_id, NativeResponseCase::SyncGamesResult(result))
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            let result = SyncBatchResultTs {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            fail_with_body(request_id, err_msg, NativeResponseCase::SyncGamesResult(result))
        }
    }
}

async fn handle_sync_dlsite_games(ctx: &AppCtx, request: &DlsiteSyncGamesRequestTs, request_id: &str) -> NativeResponseTs {
    let (input_ids, params) = to_dlsite_params(request);
    match ctx.sync_usecase.sync_dlsite_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs { success_count, error_count: 0, errors: vec![], synced_games: input_ids.clone() };
            ok(request_id, NativeResponseCase::SyncGamesResult(result))
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            let result = SyncBatchResultTs {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            fail_with_body(request_id, err_msg, NativeResponseCase::SyncGamesResult(result))
        }
    }
}

async fn handle_get_dmm_omit_works(ctx: &AppCtx, _request: &GetDmmOmitWorksRequestTs, request_id: &str) -> NativeResponseTs {
    match ctx.sync_usecase.list_dmm_omit_works().await {
        Ok(items) => {
            let list: Vec<DmmOmitWorkItemTs> = items.into_iter().map(|it| DmmOmitWorkItemTs {
                work_id: it.work_id,
                dmm: DmmOmitDmmPartTs { store_id: it.store_id, category: it.category, subcategory: it.subcategory },
            }).collect();
            ok(request_id, NativeResponseCase::DmmOmitWorks(list))
        }
        Err(e) => {
            err(request_id, anyhow_chain_to_string(&e))
        }
    }
}

fn handle_health_check(_request: &HealthCheckRequestTs, request_id: &str) -> NativeResponseTs {
    let result = HealthCheckResultTs {
        message: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    ok(request_id, NativeResponseCase::HealthCheckResult(result))
}

fn parse_message(message_bytes: &[u8]) -> HostResult<NativeMessageTs> {
    let message: NativeMessageTs = serde_json::from_slice(message_bytes)?;
    Ok(message)
}

async fn send_response_json(response: &NativeResponseTs) -> HostResult<()> {
    let mut stdout = tokio_io::stdout();

    let json_response = serde_json::to_string(&response)?;
    let json_bytes = json_response.as_bytes();
    let length = json_bytes.len() as u32;
    stdout.write_all(&length.to_le_bytes()).await?;
    stdout.write_all(json_bytes).await?;
    stdout.flush().await?;
    Ok(())
}

async fn send_error_response(request_id: &str, message: String) -> HostResult<()> {
    let response = err(request_id, message);
    send_response_json(&response).await
}

fn ok<R: Into<NativeResponseCase>>(request_id: &str, body: R) -> NativeResponseTs {
    NativeResponseTs {
        success: true,
        error: String::new(),
        request_id: request_id.to_string(),
        response: Some(body.into()),
    }
}

fn err(request_id: &str, msg: impl Into<String>) -> NativeResponseTs {
    NativeResponseTs {
        success: false,
        error: msg.into(),
        request_id: request_id.to_string(),
        response: None,
    }
}

fn fail_with_body(request_id: &str, msg: impl Into<String>, body: NativeResponseCase) -> NativeResponseTs {
    NativeResponseTs {
        success: false,
        error: msg.into(),
        request_id: request_id.to_string(),
        response: Some(body),
    }
}

fn map_egs_ts(src: &models::sync::EgsInfoTs) -> EgsInfo {
    EgsInfo {
        erogamescape_id: src.erogamescape_id,
        gamename: src.gamename.clone(),
        gamename_ruby: src.gamename_ruby.clone(),
        brandname: src.brandname.clone(),
        brandname_ruby: src.brandname_ruby.clone(),
        sellday: src.sellday.clone(),
        is_nukige: src.is_nukige,
    }
}

fn to_dmm_params(request: &DmmSyncGamesRequestTs) -> (Vec<String>, Vec<DmmSyncGameParam>) {
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
            egs: g.egs_info.as_ref().map(map_egs_ts),
            parent_pack_work_id: g.parent_pack_work_id,
        })
        .collect();
    (input_ids, params)
}

fn to_dlsite_params(request: &DlsiteSyncGamesRequestTs) -> (Vec<String>, Vec<DlsiteSyncGameParam>) {
    let input_ids: Vec<String> = request.games.iter().map(|g| g.id.clone()).collect();
    let params: Vec<DlsiteSyncGameParam> = request
        .games
        .iter()
        .map(|g| DlsiteSyncGameParam {
            store_id: g.id.clone(),
            category: g.category.clone(),
            gamename: g.title.clone(),
            image_url: g.image_url.clone(),
            egs: g.egs_info.as_ref().map(map_egs_ts),
        })
        .collect();
    (input_ids, params)
}

