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
    sqliterepository::{driver::Db as RepoDb, sqliterepository::SqliteRepositoryManager, sqliterepository::SqliteRepositories},
    image_queue_worker::ImageQueueWorker,
};
use usecase::native_host_sync::{NativeHostSyncUseCase, DmmSyncGameParam, DlsiteSyncGameParam, EgsInfo};
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

struct AppCtx {
    manager: Arc<SqliteRepositoryManager>,
    sync_usecase: NativeHostSyncUseCase<SqliteRepositoryManager, SqliteRepositories>,
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

    let resolver = Arc::new(DirsSavePathResolver::default());
    let db_path = resolver.db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repo_manager = Arc::new(SqliteRepositoryManager::new(repo_db.pool_arc()));
    let sync_usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
    let ctx = AppCtx { manager: repo_manager, sync_usecase, resolver };

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

    let _ = ctx.manager.run(|repos| {
        let log_message = format!("id: {}, request: {:?}", &message.request_id, &message.message);
        Box::pin(async move {
            repos.host_log().insert_log(HostLogLevel::Info, HostLogType::ReceiveRequest, log_message.as_str()).await?;
            Ok::<(), anyhow::Error>(())
        })
    }).await;

    let response = match &message.message {
        NativeMessageCase::SyncDmmGames(req) => handle_sync_dmm_games(ctx, req, &message.request_id).await,
        NativeMessageCase::SyncDlsiteGames(req) => handle_sync_dlsite_games(ctx, req, &message.request_id).await,
        NativeMessageCase::GetDmmOmitWorks(req) => handle_get_dmm_omit_works(ctx, req, &message.request_id).await,
        NativeMessageCase::GetStatus(_) => NativeResponseTs { success: false, error: "GetStatus is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::SetConfig(_) => NativeResponseTs { success: false, error: "SetConfig is not supported".to_string(), request_id: message.request_id.clone(), response: None },
        NativeMessageCase::HealthCheck(_) => handle_health_check(&HealthCheckRequestTs {}, &message.request_id),
    };

    send_response_json(&response).await?;

    let _ = ctx.manager.run(|repos| {
        let log_message = format!("id: {}, response: {:?}", &message.request_id, &response);
        Box::pin(async move {
            repos.host_log().insert_log(HostLogLevel::Info, HostLogType::Response, log_message.as_str()).await?;
            Ok::<(), anyhow::Error>(())
        })
    }).await;

    // 画像キューの drain は同期時のみ
    match &message.message {
        NativeMessageCase::SyncDmmGames(_) | NativeMessageCase::SyncDlsiteGames(_) => {
            let worker = ImageQueueWorker::new(ctx.manager.clone(), ctx.resolver.clone());
            let _ = worker.drain_until_empty().await;
            return Ok(false);
        }
        _ => {}
    }

    let _ = ctx.manager.run(|repos| {
        let log_message = format!("end process image queue. id: {}, message: {:?}", &message.request_id, &message.message);
        Box::pin(async move {
            repos.host_log().insert_log(HostLogLevel::Info, HostLogType::EndProcessImageQueue, log_message.as_str()).await?;
            Ok::<(), anyhow::Error>(())
        })
    }).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc as StdArc;
    use domain::repository::RepositoriesExt;
    use domain::repository::works::{DmmWorkRepository, WorkRepository};
    use domain::works::{NewDmmWork, NewWork};
    use domain::repository::manager::RepositoryManager;

    #[tokio::test]
    #[ignore]
    async fn 統合_dmm_1000件_20秒以内_半数egs_10件parent() {
        // 一時DBを用意してRepoDb経由で接続（自動マイグレーション）
        let tmp = std::env::temp_dir().join(format!("launcherg-int-{}.db3", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());

        // まず parent 用の work を1件作成し、その work_id を後続で参照
        let parent_work_id: i32 = repo_manager.run(|repos| {
            Box::pin(async move {
                // work を先に作成
                let mut work_repo = repos.work();
                let work_id = work_repo.upsert(&NewWork { title: "Parent Pack".into() }).await?.value;
                // dmm_work を紐付け
                let mut dmm = repos.dmm_work();
                let id = dmm.upsert(&NewDmmWork { store_id: "PARENT_SID".to_string(), category: "game".to_string(), subcategory: "pack".to_string(), work_id: domain::Id::new(work_id) }).await?;
                Ok::<i32, anyhow::Error>(id.value)
            })
        }).await.unwrap();

        // 1000件のDMM Workを事前投入
        let categories = ["game", "doujin"]; // 適当
        let subcategories = ["pc", "rpg", "adv", "act"]; // 適当
        repo_manager.run(|repos| {
            Box::pin(async move {
                let mut dmm = repos.dmm_work();
                for i in 0..1000 {
                    let store_id = format!("SID{:04}", i);
                    let category = categories[(i as usize) % categories.len()].to_string();
                    let subcategory = subcategories[(i as usize) % subcategories.len()].to_string();
                    // work を先に用意
                    let mut work_repo = repos.work();
                    let work_id = work_repo.upsert(&NewWork { title: format!("Game {:04}", i) }).await?.value;
                    let _ = dmm.upsert(&NewDmmWork { store_id, category, subcategory, work_id: domain::Id::new(work_id) }).await?;
                }
                Ok::<(), anyhow::Error>(())
            })
        }).await.unwrap();

        // 入力生成: 半分はEGSあり、うち10件はparent_pack_work_idを設定
        let mut params: Vec<DmmSyncGameParam> = Vec::with_capacity(1000);
        for i in 0..1000 {
            let store_id = format!("SID{:04}", i);
            let category = categories[(i as usize) % categories.len()].to_string();
            let subcategory = subcategories[(i as usize) % subcategories.len()].to_string();
            let egs = if i % 2 == 0 { Some(EgsInfo { erogamescape_id: 100000 + i, gamename: format!("EGS Name {:04}", i), gamename_ruby: "r".into(), brandname: "b".into(), brandname_ruby: "br".into(), sellday: "2024".into(), is_nukige: false }) } else { None };
            let parent = if i < 10 { Some(parent_work_id) } else { None };
            params.push(DmmSyncGameParam { store_id, category, subcategory, gamename: format!("Game {:04}", i), egs, image_url: String::new(), parent_pack_work_id: parent });
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
        assert!(elapsed.as_secs_f64() < 20.0, "1000件同期が20秒未満で終わること。実測: {:?}", elapsed);
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
        let resp = NativeResponseTs { success: false, error: "GetStatus is not supported".into(), request_id: "x".into(), response: None };
        assert!(!resp.success);
        assert!(resp.error.contains("not supported"));
    }

    #[test]
    fn セット未対応_エラー() {
        let resp = NativeResponseTs { success: false, error: "SetConfig is not supported".into(), request_id: "x".into(), response: None };
        assert!(!resp.success);
        assert!(resp.error.contains("not supported"));
    }

    #[test]
    fn dmmパラメータ変換_フィールド一致() {
        let req = DmmSyncGamesRequestTs { games: vec![models::sync::DmmGameTs {
            id: "SID1".into(), category: "game".into(), subcategory: "pc".into(), title: "T".into(),
            image_url: "u".into(), egs_info: Some(models::sync::EgsInfoTs { erogamescape_id: 1, gamename: "G".into(), gamename_ruby: "r".into(), brandname: "b".into(), brandname_ruby: "br".into(), sellday: "s".into(), is_nukige: false }), parent_pack_work_id: Some(10)
        }], extension_id: "ext".into() };
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
        let req = DlsiteSyncGamesRequestTs { games: vec![models::sync::DlsiteGameTs {
            id: "RJ1".into(), category: "doujin".into(), title: "T".into(), image_url: "u".into(),
            egs_info: None
        }], extension_id: "ext".into() };
        let (ids, params) = to_dlsite_params(&req);
        assert_eq!(ids, vec!["RJ1".to_string()]);
        assert_eq!(params[0].store_id, "RJ1");
        assert_eq!(params[0].category, "doujin");
        assert_eq!(params[0].gamename, "T");
        assert!(params[0].egs.is_none());
    }

    #[tokio::test]
    async fn 同期dmm_空入力_0件() {
        let tmp = std::env::temp_dir().join(format!("launcherg-dmm-empty-{}.db3", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let ctx = AppCtx { manager: repo_manager, sync_usecase: usecase, resolver };
        let req = DmmSyncGamesRequestTs { games: vec![], extension_id: "ext".into() };
        let resp = handle_sync_dmm_games(&ctx, &req, "r1").await;
        assert!(resp.success);
        if let Some(NativeResponseCase::SyncGamesResult(r)) = resp.response { assert_eq!(r.success_count, 0); assert!(r.synced_games.is_empty()); } else { panic!("unexpected"); }
    }

    #[tokio::test]
    async fn 同期dlsite_空入力_0件() {
        let tmp = std::env::temp_dir().join(format!("launcherg-dl-empty-{}.db3", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let ctx = AppCtx { manager: repo_manager, sync_usecase: usecase, resolver };
        let req = DlsiteSyncGamesRequestTs { games: vec![], extension_id: "ext".into() };
        let resp = handle_sync_dlsite_games(&ctx, &req, "r1").await;
        assert!(resp.success);
        if let Some(NativeResponseCase::SyncGamesResult(r)) = resp.response { assert_eq!(r.success_count, 0); assert!(r.synced_games.is_empty()); } else { panic!("unexpected"); }
    }

    #[tokio::test]
    async fn 省略作品一覧_空配列() {
        let tmp = std::env::temp_dir().join(format!("launcherg-omit-empty-{}.db3", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let tmp_str = tmp.to_string_lossy().to_string().replace("\\", "/");
        let db = RepoDb::from_path(&tmp_str).await;
        let repo_manager = StdArc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let resolver = Arc::new(DirsSavePathResolver::default());
        let usecase = NativeHostSyncUseCase::new(repo_manager.clone(), resolver.clone());
        let ctx = AppCtx { manager: repo_manager, sync_usecase: usecase, resolver };
        let resp = handle_get_dmm_omit_works(&ctx, &GetDmmOmitWorksRequestTs { extension_id: "ext".into() }, "r2").await;
        assert!(resp.success);
        match resp.response { Some(NativeResponseCase::DmmOmitWorks(v)) => assert!(v.is_empty()), _ => panic!("unexpected") }
    }
}

