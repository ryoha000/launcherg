mod models;

use chrono::Utc;
use serde_json;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{self as tokio_io, AsyncReadExt, AsyncWriteExt};

use domain::native_host_log::{HostLogLevel, HostLogType};
use domain::repository::{
    manager::RepositoryManager, native_host_log::NativeHostLogRepository, RepositoriesExt,
};
use domain::service::app_signal_router::{
    AppSignal, AppSignalEvent, AppSignalRouter, AppSignalSource,
};
use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};
use infrastructure::{
    app_signal_router::interprocess::client::InterprocessAppSignalRouter,
    heuristic_duplicate_resolver::HeuristicDuplicateResolver,
    image_queue_worker::ImageQueueWorker,
    local_file_system::LocalFileSystem,
    sqliterepository::{
        driver::Db as RepoDb, sqliterepository::SqliteRepositories,
        sqliterepository::SqliteRepositoryManager,
    },
    windowsimpl::windows::Windows,
    work_linker::WorkLinkerImpl,
};
use models::{
    common::{
        HealthCheckRequestTs, HealthCheckResultTs, NativeMessageCase, NativeMessageTs,
        NativeResponseCase, NativeResponseTs,
    },
    downloads::DownloadsCompletedRequestTs,
    packs::{DmmOmitDmmPartTs, DmmOmitWorkItemTs, GetDmmOmitWorksRequestTs},
    sync::{DlsiteSyncGamesRequestTs, DmmSyncGamesRequestTs, SyncBatchResultTs},
};
use usecase::native_host_sync::downloads::DownloadsUseCase;
use usecase::native_host_sync::{
    DlsiteSyncGameParam, DmmSyncGameParam, EgsInfo, NativeHostSyncUseCase,
};

struct AppCtx {
    manager: Arc<SqliteRepositoryManager>,
    sync_usecase: NativeHostSyncUseCase<SqliteRepositoryManager, SqliteRepositories>,
    resolver: Arc<dyn SavePathResolver>,
    fs: Arc<LocalFileSystem>,
    dedup: Arc<HeuristicDuplicateResolver>,
    work_linker: Arc<WorkLinkerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>>,
    app_signal_router: Arc<InterprocessAppSignalRouter>,
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
    err.chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ")
}

async fn log_app_signal_dispatch_failure(
    ctx: &AppCtx,
    action: impl Into<String>,
    err: anyhow::Error,
) {
    let action = action.into();
    let log_message = format!("{action}: {}", anyhow_chain_to_string(&err));
    log::warn!("app signal dispatch failed: {log_message}");

    let _ = ctx
        .manager
        .run(|repos| {
            let log_message = log_message.clone();
            Box::pin(async move {
                repos
                    .host_log()
                    .insert_log(
                        HostLogLevel::Warn,
                        HostLogType::AppSignalDispatchFailed,
                        log_message.as_str(),
                    )
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;
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

    let resolver = Arc::new(DirsSavePathResolver::default());
    // ensure sidecar (extract-icon.exe) is present next to host executable
    let _ = ensure_extract_icon_sidecar();
    let db_path = resolver.db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repo_manager = Arc::new(SqliteRepositoryManager::new(repo_db.pool_arc()));
    let sync_usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
    let fs = Arc::new(LocalFileSystem::default());
    let dedup = Arc::new(HeuristicDuplicateResolver);
    let work_linker = Arc::new(WorkLinkerImpl::new(
        repo_manager.clone(),
        resolver.clone(),
        Arc::new(Windows::new()),
    ));
    let ctx = AppCtx {
        manager: repo_manager,
        sync_usecase,
        resolver,
        fs,
        dedup,
        work_linker,
        app_signal_router: Arc::new(InterprocessAppSignalRouter::new()),
    };

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

    if let Err(err) = notify_app_signal(&ctx, &message.message).await {
        log_app_signal_dispatch_failure(&ctx, "notify_app_signal", err).await;
    }

    let _ = ctx
        .manager
        .run(|repos| {
            let log_message = format!(
                "id: {}, request: {:?}",
                &message.request_id, &message.message
            );
            Box::pin(async move {
                repos
                    .host_log()
                    .insert_log(
                        HostLogLevel::Info,
                        HostLogType::ReceiveRequest,
                        log_message.as_str(),
                    )
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;

    let response = match &message.message {
        NativeMessageCase::SyncDmmGames(req) => {
            handle_sync_dmm_games(ctx, req, &message.request_id).await
        }
        NativeMessageCase::SyncDlsiteGames(req) => {
            handle_sync_dlsite_games(ctx, req, &message.request_id).await
        }
        NativeMessageCase::GetDmmOmitWorks(req) => {
            handle_get_dmm_omit_works(ctx, req, &message.request_id).await
        }
        NativeMessageCase::DownloadsCompleted(req) => {
            handle_downloads_completed(ctx, req, &message.request_id).await
        }
        NativeMessageCase::GetStatus(_) => NativeResponseTs {
            success: false,
            error: "GetStatus is not supported".to_string(),
            request_id: message.request_id.clone(),
            response: None,
        },
        NativeMessageCase::SetConfig(_) => NativeResponseTs {
            success: false,
            error: "SetConfig is not supported".to_string(),
            request_id: message.request_id.clone(),
            response: None,
        },
        NativeMessageCase::HealthCheck(_) => {
            handle_health_check(&HealthCheckRequestTs {}, &message.request_id)
        }
    };

    send_response_json(&response).await?;

    let _ = ctx
        .manager
        .run(|repos| {
            let log_message = format!("id: {}, response: {:?}", &message.request_id, &response);
            Box::pin(async move {
                repos
                    .host_log()
                    .insert_log(
                        HostLogLevel::Info,
                        HostLogType::Response,
                        log_message.as_str(),
                    )
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;

    // 画像キューの drain は同期時のみ
    match &message.message {
        NativeMessageCase::SyncDmmGames(_) | NativeMessageCase::SyncDlsiteGames(_) => {
            // Native Messaging Host 側では HostLog を使用
            let handler = std::sync::Arc::new(
                infrastructure::image_queue_worker::handler::ImageQueueHostLogHandler::new(
                    ctx.manager.clone(),
                ),
            );
            let worker = ImageQueueWorker::new_with_event_handler(
                ctx.manager.clone(),
                ctx.resolver.clone(),
                Arc::new(Windows::new()),
                handler,
            );
            let _ = worker.drain_until_empty().await;
            return Ok(false);
        }
        _ => {}
    }

    let _ = ctx
        .manager
        .run(|repos| {
            let log_message = format!(
                "end process image queue. id: {}, message: {:?}",
                &message.request_id, &message.message
            );
            Box::pin(async move {
                repos
                    .host_log()
                    .insert_log(
                        HostLogLevel::Info,
                        HostLogType::EndProcessImageQueue,
                        log_message.as_str(),
                    )
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        })
        .await;

    Ok(true)
}
async fn handle_downloads_completed(
    ctx: &AppCtx,
    request: &DownloadsCompletedRequestTs,
    request_id: &str,
) -> NativeResponseTs {
    let usecase: DownloadsUseCase<
        SqliteRepositoryManager,
        SqliteRepositories,
        LocalFileSystem,
        HeuristicDuplicateResolver,
        WorkLinkerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>,
    > = DownloadsUseCase::new(
        ctx.manager.clone(),
        ctx.resolver.clone(),
        ctx.fs.clone(),
        ctx.dedup.clone(),
        ctx.work_linker.clone(),
    );

    // helper: resolve work_id from intent (DMM / DLsite)
    let work_id = match &request.intent {
        models::downloads::DownloadIntentTs::Dmm { game_store_id, .. } => {
            match usecase.resolve_dmm_work_id(game_store_id).await {
                Ok(v) => v,
                Err(e) => {
                    let msg = e.to_string();
                    if let Err(dispatch_err) =
                        dispatch_show_error_message(&ctx.app_signal_router, msg.clone()).await
                    {
                        log_app_signal_dispatch_failure(
                            &ctx,
                            format!("dispatch_show_error_message request_id={}", request_id),
                            dispatch_err,
                        )
                        .await;
                    }
                    return err(request_id, msg);
                }
            }
        }
        models::downloads::DownloadIntentTs::Dlsite { game_store_id, .. } => {
            match usecase.resolve_dlsite_work_id(game_store_id).await {
                Ok(v) => v,
                Err(e) => {
                    let msg = e.to_string();
                    if let Err(dispatch_err) =
                        dispatch_show_error_message(&ctx.app_signal_router, msg.clone()).await
                    {
                        log_app_signal_dispatch_failure(
                            &ctx,
                            format!("dispatch_show_error_message request_id={}", request_id),
                            dispatch_err,
                        )
                        .await;
                    }
                    return err(request_id, msg);
                }
            }
        }
    };

    // items == 1
    if request.items.len() == 1 {
        let item = &request.items[0];
        if let Err(e) = usecase.handle_single(&item.filename, work_id.clone()).await {
            let msg = e.to_string();
            if let Err(dispatch_err) =
                dispatch_show_error_message(&ctx.app_signal_router, msg.clone()).await
            {
                log_app_signal_dispatch_failure(
                    &ctx,
                    format!("dispatch_show_error_message request_id={}", request_id),
                    dispatch_err,
                )
                .await;
            }
            return err(request_id, msg);
        }
        cleanup_download_paths([item.filename.as_str()]);
        if let Err(err) = dispatch_refetch_work(&ctx.app_signal_router, work_id.value).await {
            log_app_signal_dispatch_failure(
                &ctx,
                format!("dispatch_refetch_work work_id={}", work_id.value),
                err,
            )
            .await;
        }
        return ok(
            request_id,
            NativeResponseCase::HealthCheckResult(HealthCheckResultTs {
                message: "OK".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            }),
        );
    }

    // items >= 2 (split)
    if request.items.len() >= 2 {
        let paths: Vec<String> = request.items.iter().map(|i| i.filename.clone()).collect();
        if let Err(e) = usecase.handle_split(&paths, work_id.clone()).await {
            let msg = e.to_string();
            if let Err(dispatch_err) =
                dispatch_show_error_message(&ctx.app_signal_router, msg.clone()).await
            {
                log_app_signal_dispatch_failure(
                    &ctx,
                    format!("dispatch_show_error_message request_id={}", request_id),
                    dispatch_err,
                )
                .await;
            }
            return err(request_id, msg);
        }
        cleanup_download_paths(paths.iter().map(|p| p.as_str()));
        if let Err(err) = dispatch_refetch_work(&ctx.app_signal_router, work_id.value).await {
            log_app_signal_dispatch_failure(
                &ctx,
                format!("dispatch_refetch_work work_id={}", work_id.value),
                err,
            )
            .await;
        }
        return ok(
            request_id,
            NativeResponseCase::HealthCheckResult(HealthCheckResultTs {
                message: "OK".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            }),
        );
    }

    if let Err(err) = dispatch_refetch_work(&ctx.app_signal_router, work_id.value).await {
        log_app_signal_dispatch_failure(
            &ctx,
            format!("dispatch_refetch_work work_id={}", work_id.value),
            err,
        )
        .await;
    }

    ok(
        request_id,
        NativeResponseCase::HealthCheckResult(HealthCheckResultTs {
            message: "NOOP".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }),
    )
}

async fn notify_app_signal(ctx: &AppCtx, message: &NativeMessageCase) -> anyhow::Result<()> {
    let message = match message {
        NativeMessageCase::SyncDmmGames(req) => Some(format!(
            "DMM GAMES との連携リクエストを受信しました（対象 {} 件）。処理が完了すると画面が自動で更新されます。",
            req.games.len()
        )),
        NativeMessageCase::SyncDlsiteGames(req) => Some(format!(
            "DLsite との連携リクエストを受信しました（対象 {} 件）。処理が完了すると画面が自動で更新されます。",
            req.games.len()
        )),
        NativeMessageCase::DownloadsCompleted(req) => {
            let store_label = match &req.intent {
                models::downloads::DownloadIntentTs::Dmm { .. } => "DMM GAMES",
                models::downloads::DownloadIntentTs::Dlsite { .. } => "DLsite",
            };
            Some(format!(
                "{} のダウンロード完了情報を受信しました（対象 {} 件）。処理が完了すると画面が自動で更新されます。",
                store_label,
                req.items.len()
            ))
        }
        _ => None,
    };

    if let Some(message) = message {
        dispatch_show_message(&ctx.app_signal_router, message).await?;
    }

    Ok(())
}

async fn dispatch_show_message(
    router: &Arc<InterprocessAppSignalRouter>,
    message: String,
) -> anyhow::Result<()> {
    let signal = AppSignal {
        source: AppSignalSource::NativeMessagingHost,
        event: AppSignalEvent::ShowMessage { message },
        issued_at: Utc::now(),
    };
    router.dispatch(signal).await
}

async fn dispatch_show_error_message(
    router: &Arc<InterprocessAppSignalRouter>,
    message: String,
) -> anyhow::Result<()> {
    let signal = AppSignal {
        source: AppSignalSource::NativeMessagingHost,
        event: AppSignalEvent::ShowErrorMessage { message },
        issued_at: Utc::now(),
    };
    router.dispatch(signal).await
}

async fn dispatch_refetch_work(
    router: &Arc<InterprocessAppSignalRouter>,
    work_id: i32,
) -> anyhow::Result<()> {
    let signal = AppSignal {
        source: AppSignalSource::NativeMessagingHost,
        event: AppSignalEvent::RefetchWork { work_id },
        issued_at: Utc::now(),
    };
    router.dispatch(signal).await
}

async fn dispatch_refetch_works(router: &Arc<InterprocessAppSignalRouter>) -> anyhow::Result<()> {
    let signal = AppSignal {
        source: AppSignalSource::NativeMessagingHost,
        event: AppSignalEvent::RefetchWorks,
        issued_at: Utc::now(),
    };
    router.dispatch(signal).await
}

fn cleanup_download_paths<I>(paths: I)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    for item in paths {
        let path_str = item.as_ref();
        let path = Path::new(path_str);
        match fs::metadata(path) {
            Ok(metadata) => {
                let remove_result = if metadata.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };
                if let Err(e) = remove_result {
                    if e.kind() != ErrorKind::NotFound {
                        log::warn!(
                            "failed to remove downloaded item: {} ({})",
                            path.display(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    log::warn!(
                        "failed to inspect downloaded item: {} ({})",
                        path.display(),
                        e
                    );
                }
            }
        }
    }
}

async fn handle_sync_dmm_games(
    ctx: &AppCtx,
    request: &DmmSyncGamesRequestTs,
    request_id: &str,
) -> NativeResponseTs {
    let (input_ids, params) = to_dmm_params(request);
    match ctx.sync_usecase.sync_dmm_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs {
                success_count,
                error_count: 0,
                errors: vec![],
                synced_games: input_ids.clone(),
            };
            if let Err(err) = dispatch_refetch_works(&ctx.app_signal_router).await {
                log_app_signal_dispatch_failure(&ctx, "dispatch_refetch_works", err).await;
            }
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
            if let Err(dispatch_err) =
                dispatch_show_error_message(&ctx.app_signal_router, err_msg.clone()).await
            {
                log_app_signal_dispatch_failure(
                    &ctx,
                    format!("dispatch_show_error_message request_id={}", request_id),
                    dispatch_err,
                )
                .await;
            }
            fail_with_body(
                request_id,
                err_msg,
                NativeResponseCase::SyncGamesResult(result),
            )
        }
    }
}

async fn handle_sync_dlsite_games(
    ctx: &AppCtx,
    request: &DlsiteSyncGamesRequestTs,
    request_id: &str,
) -> NativeResponseTs {
    let (input_ids, params) = to_dlsite_params(request);
    match ctx.sync_usecase.sync_dlsite_games(params).await {
        Ok(success_count) => {
            let result = SyncBatchResultTs {
                success_count,
                error_count: 0,
                errors: vec![],
                synced_games: input_ids.clone(),
            };
            if let Err(err) = dispatch_refetch_works(&ctx.app_signal_router).await {
                log_app_signal_dispatch_failure(&ctx, "dispatch_refetch_works", err).await;
            }
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
            if let Err(dispatch_err) =
                dispatch_show_error_message(&ctx.app_signal_router, err_msg.clone()).await
            {
                log_app_signal_dispatch_failure(
                    &ctx,
                    format!("dispatch_show_error_message request_id={}", request_id),
                    dispatch_err,
                )
                .await;
            }
            fail_with_body(
                request_id,
                err_msg,
                NativeResponseCase::SyncGamesResult(result),
            )
        }
    }
}

async fn handle_get_dmm_omit_works(
    ctx: &AppCtx,
    _request: &GetDmmOmitWorksRequestTs,
    request_id: &str,
) -> NativeResponseTs {
    match ctx.sync_usecase.list_dmm_omit_works().await {
        Ok(items) => {
            let list: Vec<DmmOmitWorkItemTs> = items
                .into_iter()
                .map(|it| DmmOmitWorkItemTs {
                    work_id: it.work_id,
                    dmm: DmmOmitDmmPartTs {
                        store_id: it.store_id,
                        category: it.category,
                        subcategory: it.subcategory,
                    },
                })
                .collect();
            ok(request_id, NativeResponseCase::DmmOmitWorks(list))
        }
        Err(e) => err(request_id, anyhow_chain_to_string(&e)),
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

fn ensure_extract_icon_sidecar() -> anyhow::Result<PathBuf> {
    // resolve destination: same directory as current executable
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or_else(|| anyhow::anyhow!("failed to resolve current exe directory"))?;
    let dst = exe_dir.join("extract-icon.exe");
    if dst.exists() {
        return Ok(dst);
    }
    // embedded bytes from repository's prebuilt binary
    const BYTES: &[u8] = include_bytes!("../../bin/extract-icon-x86_64-pc-windows-msvc.exe");
    fs::write(&dst, BYTES)?;
    Ok(dst)
}

fn fail_with_body(
    request_id: &str,
    msg: impl Into<String>,
    body: NativeResponseCase,
) -> NativeResponseTs {
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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::collection::CollectionRepository;
    use domain::repository::manager::RepositoryManager;
    use domain::repository::works::{DmmWorkRepository, WorkRepository};
    use domain::repository::RepositoriesExt;
    use domain::works::{NewDmmWork, NewWork, WorkDetails};
    use rand::Rng;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc as StdArc;

    async fn setup_db() -> RepoDb {
        let rng = rand::rng();
        let suffix: String = rng
            .sample_iter(rand::distr::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let tmp = std::env::temp_dir().join(format!("launcherg-int-{}.db3", suffix));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        RepoDb::from_path(&tmp_str).await
    }

    #[tokio::test]
    #[ignore]
    async fn 統合_dmm_1000件_20秒以内_半数egs_10件parent() {
        let db = setup_db().await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());

        // まず parent 用の work を1件作成し、その work_id を後続で参照
        let parent_work_id: i32 = repo_manager
            .run(|repos| {
                Box::pin(async move {
                    // work を先に作成
                    let mut work_repo = repos.work();
                    let work_id = work_repo
                        .upsert(&NewWork {
                            title: "Parent Pack".into(),
                        })
                        .await?
                        .value;
                    // dmm_work を紐付け
                    let mut dmm = repos.dmm_work();
                    let id = dmm
                        .upsert(&NewDmmWork {
                            store_id: "PARENT_SID".to_string(),
                            category: "game".to_string(),
                            subcategory: "pack".to_string(),
                            work_id: domain::Id::new(work_id),
                        })
                        .await?;
                    Ok::<i32, anyhow::Error>(id.value)
                })
            })
            .await
            .unwrap();

        // 1000件のDMM Workを事前投入
        let categories = ["game", "doujin"]; // 適当
        let subcategories = ["pc", "rpg", "adv", "act"]; // 適当
        repo_manager
            .run(|repos| {
                Box::pin(async move {
                    let mut dmm = repos.dmm_work();
                    for i in 0..1000 {
                        let store_id = format!("SID{:04}", i);
                        let category = categories[(i as usize) % categories.len()].to_string();
                        let subcategory =
                            subcategories[(i as usize) % subcategories.len()].to_string();
                        // work を先に用意
                        let mut work_repo = repos.work();
                        let work_id = work_repo
                            .upsert(&NewWork {
                                title: format!("Game {:04}", i),
                            })
                            .await?
                            .value;
                        let _ = dmm
                            .upsert(&NewDmmWork {
                                store_id,
                                category,
                                subcategory,
                                work_id: domain::Id::new(work_id),
                            })
                            .await?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
            .unwrap();

        // 入力生成: 半分はEGSあり、うち10件はparent_pack_work_idを設定
        let mut params: Vec<DmmSyncGameParam> = Vec::with_capacity(1000);
        for i in 0..1000 {
            let store_id = format!("SID{:04}", i);
            let category = categories[(i as usize) % categories.len()].to_string();
            let subcategory = subcategories[(i as usize) % subcategories.len()].to_string();
            let egs = if i % 2 == 0 {
                Some(EgsInfo {
                    erogamescape_id: 100000 + i,
                    gamename: format!("EGS Name {:04}", i),
                    gamename_ruby: "r".into(),
                    brandname: "b".into(),
                    brandname_ruby: "br".into(),
                    sellday: "2024".into(),
                    is_nukige: false,
                })
            } else {
                None
            };
            let parent = if i < 10 { Some(parent_work_id) } else { None };
            params.push(DmmSyncGameParam {
                store_id,
                category,
                subcategory,
                gamename: format!("Game {:04}", i),
                egs,
                image_url: String::new(),
                parent_pack_work_id: parent,
            });
        }

        // 実行と時間計測
        use std::time::Instant;
        let start = Instant::now();
        println!("start sync");
        let synced = usecase.sync_dmm_games(params).await.unwrap();
        let elapsed = start.elapsed();
        println!("synced: {}", synced);
        println!("elapsed: {:?}", elapsed);
        assert_eq!(synced, 1000, "同期件数が一致すること");
        assert!(
            elapsed.as_secs_f64() < 20.0,
            "1000件同期が20秒未満で終わること。実測: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn 統合_dmm_dlsite() {
        // 以下のケースについてテスト
        // 1. DMM でのみ管理
        // 2. DLsite でのみ管理
        // 3. DMM と DLsite で同じものが管理されている(DMM が既存)
        // 4. DMM と DLsite で同じものが管理されている(DLsite が既存)
        // 5. collection_element に紐づいているときに DMM の登録(erogamescape_id が既存)
        // 6. collection_element に紐づいているときに DLsite の登録(erogamescape_id が既存)

        enum SyncGameParam {
            Dmm(Vec<DmmSyncGameParam>),
            Dlsite(Vec<DlsiteSyncGameParam>),
        }

        struct TestCase {
            pub name: String,
            pub params: Vec<SyncGameParam>,
            pub setup: Box<
                dyn Fn(Arc<SqliteRepositoryManager>) -> Pin<Box<dyn Future<Output = ()> + Send>>
                    + Send
                    + Sync,
            >,
            pub assertions: Box<dyn Fn(Vec<WorkDetails>) -> bool + Send + Sync>,
        }

        let egs = EgsInfo {
            erogamescape_id: 1,
            gamename: "Game 1".to_string(),
            gamename_ruby: "ゲームイチ".to_string(),
            brandname: "りょはそふと".to_string(),
            brandname_ruby: "リョハソフト".to_string(),
            sellday: "2025-01-01".to_string(),
            is_nukige: false,
        };

        let dmm_param_egs = DmmSyncGameParam {
            store_id: "SID1".to_string(),
            category: "game".to_string(),
            subcategory: "pc".to_string(),
            gamename: "Game 1".to_string(),
            egs: Some(egs.clone()),
            image_url: "https://example.com/image1_ps.jpg".to_string(),
            parent_pack_work_id: None,
        };
        let dmm_param = DmmSyncGameParam {
            store_id: "SID2".to_string(),
            category: "game".to_string(),
            subcategory: "pc".to_string(),
            gamename: "Game 2".to_string(),
            egs: None,
            image_url: "https://example.com/image2_ps.jpg".to_string(),
            parent_pack_work_id: None,
        };

        let dlsite_param_egs = DlsiteSyncGameParam {
            store_id: "RJ1".to_string(),
            category: "doujin".to_string(),
            gamename: "Game 1".to_string(),
            egs: Some(egs.clone()),
            image_url: "https://example.com/resize/images2/image1.jpg".to_string(),
        };
        let dlsite_param = DlsiteSyncGameParam {
            store_id: "RJ2".to_string(),
            category: "doujin".to_string(),
            gamename: "Game 2".to_string(),
            egs: None,
            image_url: "https://example.com/resize/images2/image2.jpg".to_string(),
        };

        let test_cases = vec![
            TestCase {
                name: "DMM でのみ管理".to_string(),
                params: vec![SyncGameParam::Dmm(vec![
                    dmm_param_egs.clone(),
                    dmm_param.clone(),
                ])],
                setup: Box::new(|_| Box::pin(async move { () })),
                assertions: Box::new(|works| {
                    // 2件存在し、両方 dmm に紐づく、SID1 は EGS が紐づいている、SID2 は EGS が紐づいていない
                    assert_eq!(works.len(), 2);
                    for work in works {
                        assert!(work.dlsite.is_none());
                        assert!(work.dmm.is_some());
                        let dmm = work.dmm.unwrap();
                        if dmm.store_id == "SID1" {
                            assert!(work.erogamescape.is_some());
                        } else if dmm.store_id == "SID2" {
                            assert!(work.erogamescape.is_none());
                        } else {
                            assert!(false, "unexpected store_id: {}", dmm.store_id);
                        }
                    }
                    true
                }),
            },
            TestCase {
                name: "DLsite でのみ管理".to_string(),
                params: vec![SyncGameParam::Dlsite(vec![
                    dlsite_param_egs.clone(),
                    dlsite_param.clone(),
                ])],
                setup: Box::new(|_| Box::pin(async move { () })),
                assertions: Box::new(|works| {
                    // 2件存在し、両方 dlsite に紐づく、RJ1 は EGS が紐づいている、RJ2 は EGS が紐づいていない
                    assert_eq!(works.len(), 2);
                    for work in works {
                        assert!(work.dmm.is_none());
                        assert!(work.dlsite.is_some());
                        let dlsite = work.dlsite.unwrap();
                        if dlsite.store_id == "RJ1" {
                            assert!(work.erogamescape.is_some());
                        } else if dlsite.store_id == "RJ2" {
                            assert!(work.erogamescape.is_none());
                        } else {
                            assert!(false, "unexpected store_id: {}", dlsite.store_id);
                        }
                    }
                    true
                }),
            },
            TestCase {
                name: "DMM と DLsite で同じものが管理されている(DMM が既存)".to_string(),
                params: vec![
                    SyncGameParam::Dmm(vec![dmm_param_egs.clone()]),
                    SyncGameParam::Dlsite(vec![dlsite_param_egs.clone()]),
                ],
                setup: Box::new(|_| Box::pin(async move { () })),
                assertions: Box::new(|works| {
                    // 1件存在し、dmm, dlsite, egs が紐づく
                    assert_eq!(works.len(), 1);
                    for work in works {
                        assert!(work.dmm.is_some());
                        assert!(work.dlsite.is_some());
                        assert!(work.erogamescape.is_some());
                    }
                    true
                }),
            },
            TestCase {
                name: "DMM と DLsite で同じものが管理されている(DLsite が既存)".to_string(),
                params: vec![
                    SyncGameParam::Dlsite(vec![dlsite_param_egs.clone()]),
                    SyncGameParam::Dmm(vec![dmm_param_egs.clone()]),
                ],
                setup: Box::new(|_| Box::pin(async move { () })),
                assertions: Box::new(|works| {
                    // 1件存在し、dmm, dlsite, egs が紐づく
                    assert_eq!(works.len(), 1);
                    for work in works {
                        assert!(work.dmm.is_some());
                        assert!(work.dlsite.is_some());
                        assert!(work.erogamescape.is_some());
                    }
                    true
                }),
            },
            TestCase {
                name: "collection_element に紐づいているときに DMM の登録(erogamescape_id が既存)"
                    .to_string(),
                params: vec![SyncGameParam::Dmm(vec![dmm_param_egs.clone()])],
                setup: Box::new(|manager| {
                    Box::pin(async move {
                        manager
                            .run(|repos| {
                                Box::pin(async move {
                                    let cid = repos
                                        .collection()
                                        .allocate_new_collection_element_id("Game 1")
                                        .await?;
                                    repos.collection().upsert_erogamescape_map(&cid, 1).await?;

                                    Ok(())
                                })
                            })
                            .await
                            .unwrap();
                    })
                }),
                assertions: Box::new(|works| {
                    // 1件存在し、dmm, egs が紐づく
                    assert_eq!(works.len(), 1);
                    for work in works {
                        assert!(work.dmm.is_some());
                        assert!(work.dlsite.is_none());
                        assert!(work.erogamescape.is_some());
                    }
                    true
                }),
            },
            TestCase {
                name:
                    "collection_element に紐づいているときに DLsite の登録(erogamescape_id が既存)"
                        .to_string(),
                params: vec![SyncGameParam::Dlsite(vec![dlsite_param_egs.clone()])],
                setup: Box::new(|manager| {
                    Box::pin(async move {
                        manager
                            .run(|repos| {
                                Box::pin(async move {
                                    let cid = repos
                                        .collection()
                                        .allocate_new_collection_element_id("Game 1")
                                        .await?;
                                    repos.collection().upsert_erogamescape_map(&cid, 1).await?;

                                    Ok(())
                                })
                            })
                            .await
                            .unwrap();
                    })
                }),
                assertions: Box::new(|works| {
                    // 1件存在し、dlsite, egs が紐づく
                    assert_eq!(works.len(), 1);
                    for work in works {
                        assert!(work.dlsite.is_some());
                        assert!(work.erogamescape.is_some());
                        assert!(work.dmm.is_none());
                    }
                    true
                }),
            },
        ];

        for test_case in test_cases {
            let db = setup_db().await;
            let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
            let resolver = Arc::new(DirsSavePathResolver::default());
            let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
            (test_case.setup)(repo_manager.clone()).await;

            // 同じDBを使いまわしていないか確認
            let works = repo_manager
                .run(|repos| {
                    Box::pin(async move {
                        let works = repos.work().list_all_details().await?;
                        Ok::<Vec<WorkDetails>, anyhow::Error>(works)
                    })
                })
                .await
                .unwrap();
            assert!(works.is_empty());

            for param in test_case.params {
                match param {
                    SyncGameParam::Dmm(params) => {
                        usecase.sync_dmm_games(params).await.unwrap();
                    }
                    SyncGameParam::Dlsite(params) => {
                        usecase.sync_dlsite_games(params).await.unwrap();
                    }
                }
            }
            let works = repo_manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut work_repo = repos.work();
                        work_repo.list_all_details().await
                    })
                })
                .await
                .unwrap();
            assert!((test_case.assertions)(works), "{}", test_case.name);
        }
    }

    #[test]
    fn ヘルスチェック_okが返る() {
        let resp = handle_health_check(&HealthCheckRequestTs {}, "req1");
        assert!(resp.success);
        match resp.response {
            Some(NativeResponseCase::HealthCheckResult(body)) => {
                assert_eq!(body.message, "OK");
                assert!(!body.version.is_empty());
            }
            _ => panic!("unexpected response"),
        }
    }

    #[test]
    fn ステータス未対応_エラー() {
        let resp = NativeResponseTs {
            success: false,
            error: "GetStatus is not supported".into(),
            request_id: "x".into(),
            response: None,
        };
        assert!(!resp.success);
        assert!(resp.error.contains("not supported"));
    }

    #[test]
    fn セット未対応_エラー() {
        let resp = NativeResponseTs {
            success: false,
            error: "SetConfig is not supported".into(),
            request_id: "x".into(),
            response: None,
        };
        assert!(!resp.success);
        assert!(resp.error.contains("not supported"));
    }

    #[test]
    fn dmmパラメータ変換_フィールド一致() {
        let req = DmmSyncGamesRequestTs {
            games: vec![models::sync::DmmGameTs {
                id: "SID1".into(),
                category: "game".into(),
                subcategory: "pc".into(),
                title: "T".into(),
                image_url: "u".into(),
                egs_info: Some(models::sync::EgsInfoTs {
                    erogamescape_id: 1,
                    gamename: "G".into(),
                    gamename_ruby: "r".into(),
                    brandname: "b".into(),
                    brandname_ruby: "br".into(),
                    sellday: "s".into(),
                    is_nukige: false,
                }),
                parent_pack_work_id: Some(10),
            }],
            extension_id: "ext".into(),
        };
        let (ids, params) = to_dmm_params(&req);
        assert_eq!(ids, vec!["SID1".to_string()]);
        assert_eq!(params[0].store_id, "SID1");
        assert_eq!(params[0].category, "game");
        assert_eq!(params[0].subcategory, "pc");
        assert_eq!(params[0].gamename, "T");
        assert!(params[0].egs.is_some());
        assert_eq!(params[0].parent_pack_work_id, Some(10));
    }

    #[test]
    fn dlsiteパラメータ変換_フィールド一致() {
        let req = DlsiteSyncGamesRequestTs {
            games: vec![models::sync::DlsiteGameTs {
                id: "RJ1".into(),
                category: "doujin".into(),
                title: "T".into(),
                image_url: "u".into(),
                egs_info: None,
            }],
            extension_id: "ext".into(),
        };
        let (ids, params) = to_dlsite_params(&req);
        assert_eq!(ids, vec!["RJ1".to_string()]);
        assert_eq!(params[0].store_id, "RJ1");
        assert_eq!(params[0].category, "doujin");
        assert_eq!(params[0].gamename, "T");
        assert!(params[0].egs.is_none());
    }

    #[tokio::test]
    async fn 同期dmm_空入力_0件() {
        let tmp = std::env::temp_dir().join(format!(
            "launcherg-dmm-empty-{}.db3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let fs = Arc::new(LocalFileSystem::default());
        let dedup = Arc::new(HeuristicDuplicateResolver);
        let work_linker = Arc::new(WorkLinkerImpl::new(
            repo_manager.clone(),
            resolver.clone(),
            Arc::new(Windows::new()),
        ));
        let ctx = AppCtx {
            manager: repo_manager,
            sync_usecase: usecase,
            resolver,
            fs,
            dedup,
            work_linker,
            app_signal_router: Arc::new(InterprocessAppSignalRouter::new()),
        };
        let req = DmmSyncGamesRequestTs {
            games: vec![],
            extension_id: "ext".into(),
        };
        let resp = handle_sync_dmm_games(&ctx, &req, "r1").await;
        assert!(resp.success);
        if let Some(NativeResponseCase::SyncGamesResult(r)) = resp.response {
            assert_eq!(r.success_count, 0);
            assert!(r.synced_games.is_empty());
        } else {
            panic!("unexpected");
        }
    }

    #[tokio::test]
    async fn 同期dlsite_空入力_0件() {
        let tmp = std::env::temp_dir().join(format!(
            "launcherg-dl-empty-{}.db3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let fs = Arc::new(LocalFileSystem::default());
        let dedup = Arc::new(HeuristicDuplicateResolver);
        let work_linker = Arc::new(WorkLinkerImpl::new(
            repo_manager.clone(),
            resolver.clone(),
            Arc::new(Windows::new()),
        ));
        let ctx = AppCtx {
            manager: repo_manager,
            sync_usecase: usecase,
            resolver,
            fs,
            dedup,
            work_linker,
            app_signal_router: Arc::new(InterprocessAppSignalRouter::new()),
        };
        let req = DlsiteSyncGamesRequestTs {
            games: vec![],
            extension_id: "ext".into(),
        };
        let resp = handle_sync_dlsite_games(&ctx, &req, "r1").await;
        assert!(resp.success);
        if let Some(NativeResponseCase::SyncGamesResult(r)) = resp.response {
            assert_eq!(r.success_count, 0);
            assert!(r.synced_games.is_empty());
        } else {
            panic!("unexpected");
        }
    }

    #[tokio::test]
    async fn 省略作品一覧_空配列() {
        let tmp = std::env::temp_dir().join(format!(
            "launcherg-omit-empty-{}.db3",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let fs = Arc::new(LocalFileSystem::default());
        let dedup = Arc::new(HeuristicDuplicateResolver);
        let work_linker = Arc::new(WorkLinkerImpl::new(
            repo_manager.clone(),
            resolver.clone(),
            Arc::new(Windows::new()),
        ));
        let ctx = AppCtx {
            manager: repo_manager,
            sync_usecase: usecase,
            resolver,
            fs,
            dedup,
            work_linker,
            app_signal_router: Arc::new(InterprocessAppSignalRouter::new()),
        };
        let resp = handle_get_dmm_omit_works(
            &ctx,
            &GetDmmOmitWorksRequestTs {
                extension_id: "ext".into(),
            },
            "r2",
        )
        .await;
        assert!(resp.success);
        match resp.response {
            Some(NativeResponseCase::DmmOmitWorks(v)) => assert!(v.is_empty()),
            _ => panic!("unexpected"),
        }
    }
}
