mod proto;
#[path = "../infrastructure/mod.rs"]
mod infrastructure;
#[path = "../usecase/mod.rs"]
mod usecase;
#[path = "../domain/mod.rs"]
mod domain;

use std::io::ErrorKind;
use std::sync::Arc;
use tokio::io::{self as tokio_io, AsyncReadExt, AsyncWriteExt};
use prost::Message;
 
use serde_json;
use serde::Deserialize;
use thiserror::Error;
 

// プロトタイプを使用
use proto::generated::launcherg::{common::*, sync::*, status::*};
use infrastructure::{
    repositoryimpl::{driver::Db as RepoDb, repository::Repositories},
    thumbnail::ThumbnailServiceImpl,
    icon::IconServiceImpl,
};
use usecase::native_host_sync::{NativeHostSyncUseCase, DmmSyncGameParam, DlsiteSyncGameParam, EgsInfo};

struct AppCtx {
    sync_usecase: NativeHostSyncUseCase<Repositories, ThumbnailServiceImpl, IconServiceImpl>,
}

#[derive(Debug)]
enum RequestFormat {
    Protobuf,
    Json,
    JsonBuf,
}

// 共通エラー型（最小導入）
type HostResult<T> = Result<T, HostError>;

#[derive(Debug, Error)]
enum HostError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Message too large: {0} bytes (limit 1048576)")]
    TooLarge(usize),
    #[error("Protocol error: {0}")]
    Protocol(String),
}

// エラーチェーンを1行の文字列に整形
fn error_chain_to_string<E: std::error::Error + ?Sized>(err: &E) -> String {
    let mut msgs = vec![err.to_string()];
    let mut curr = err.source();
    while let Some(src) = curr {
        msgs.push(src.to_string());
        curr = src.source();
    }
    msgs.join(": ")
}

// anyhow::Error 専用（chain() が使えるため安全）
fn anyhow_chain_to_string(err: &anyhow::Error) -> String {
    err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(": ")
}

// 4バイト長 + 本体のフレーミングを読み取り（None=EOF）
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

// 統一的なエラーレスポンス送信
async fn send_error_response(request_id: &str, message: String, preferred_format: RequestFormat) -> Result<(), Box<dyn std::error::Error>> {
    let response = NativeResponse {
        success: false,
        error: message,
        request_id: request_id.to_string(),
        response: None,
    };
    send_response_with_format(&response, preferred_format).await
}

#[tokio::main]
async fn main() {
    // 標準エラー出力にログを記録
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    // UseCase/Repository を初期化
    let db_path = usecase::native_host_sync::db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repositories = Repositories::new(repo_db);
    let host_root = crate::usecase::native_host_sync::host_root_dir();
    let thumbs = ThumbnailServiceImpl::new(host_root.clone());
    let icons = IconServiceImpl::new_from_root_path(host_root);
    let sync_usecase = NativeHostSyncUseCase::new(Arc::new(repositories), Arc::new(thumbs), Arc::new(icons));
    let ctx = AppCtx { sync_usecase };

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
    // フレームを読み取り
    let message_bytes = match read_framed().await {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(false),
        Err(HostError::TooLarge(length)) => {
            let error_msg = format!("Message too large: {} bytes (limit 1048576)", length);
            // 形式は拡張が既定で使用するBuf JSONに合わせる
            send_error_response("", error_msg, RequestFormat::JsonBuf).await?;
            return Ok(true);
        }
        Err(e) => return Err(e.into()),
    };
    
    // リクエスト形式を判定し、メッセージをパース
    let (message, format) = match parse_message(&message_bytes) {
        Ok(v) => v,
        Err(err) => {
            // パースできなかった場合も、可能なら requestId を抽出してエラーを返す
            let mut request_id = String::new();
            if let Ok(s) = std::str::from_utf8(&message_bytes) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                    if let Some(id) = v.get("requestId").and_then(|x| x.as_str()) {
                        request_id = id.to_string();
                    }
                }
            }

            send_error_response(&request_id, err.to_string(), RequestFormat::JsonBuf).await?;
            return Ok(true);
        }
    };
    
    let message_type = match &message.message {
        Some(native_message::Message::SyncDmmGames(_)) => "sync_dmm_games",
        Some(native_message::Message::SyncDlsiteGames(_)) => "sync_dlsite_games",
        Some(native_message::Message::GetStatus(_)) => "get_status",
        Some(native_message::Message::SetConfig(_)) => "set_config",
        Some(native_message::Message::HealthCheck(_)) => "health_check",
        None => "unknown",
    };
    log::info!("Received message type: {}", message_type);
    
    // メッセージタイプに応じて処理
    let response = match &message.message {
        Some(native_message::Message::SyncDmmGames(req)) => handle_sync_dmm_games(ctx, req, &message.request_id).await,
        Some(native_message::Message::SyncDlsiteGames(req)) => handle_sync_dlsite_games(ctx, req, &message.request_id).await,
        Some(native_message::Message::GetStatus(req)) => handle_get_status(req, &message.request_id),
        Some(native_message::Message::SetConfig(req)) => handle_set_config(req, &message.request_id),
        Some(native_message::Message::HealthCheck(req)) => handle_health_check(req, &message.request_id),
        None => NativeResponse {
            success: false,
            error: "No message content provided".to_string(),
            request_id: message.request_id.clone(),
            response: None,
        },
    };
    
    // レスポンス形式に応じて送信
    send_response_with_format(&response, format).await?;
    
    Ok(true)
}

async fn handle_sync_dmm_games(ctx: &AppCtx, request: &DmmSyncGamesRequest, request_id: &str) -> NativeResponse {
    log::info!("Syncing {} DMM games", request.games.len());
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
            usecase::native_host_sync::bump_sync_counters(success_count);
            let result = SyncBatchResult { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::SyncGamesResult(result)) }
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            log::error!("sync_dmm_games failed: {}", err_msg);

            let result = SyncBatchResult {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            NativeResponse {
                success: false,
                error: err_msg,
                request_id: request_id.to_string(),
                response: Some(native_response::Response::SyncGamesResult(result)),
            }
        }
    }
}

async fn handle_sync_dlsite_games(ctx: &AppCtx, request: &DlsiteSyncGamesRequest, request_id: &str) -> NativeResponse {
    log::info!("Syncing {} DLsite games", request.games.len());
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
            usecase::native_host_sync::bump_sync_counters(success_count);
            let result = SyncBatchResult { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::SyncGamesResult(result)) }
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            log::error!("sync_dlsite_games failed: {}", err_msg);

            let result = SyncBatchResult {
                success_count: 0,
                error_count: input_ids.len() as u32,
                errors: vec![err_msg.clone()],
                synced_games: input_ids,
            };
            NativeResponse {
                success: false,
                error: err_msg,
                request_id: request_id.to_string(),
                response: Some(native_response::Response::SyncGamesResult(result)),
            }
        }
    }
}

fn handle_get_status(_request: &GetStatusRequest, request_id: &str) -> NativeResponse {
    let data = usecase::native_host_sync::get_status_data();
    let status = SyncStatus {
        last_sync: data.last_sync_seconds.map(|sec| pbjson_types::Timestamp { seconds: sec, nanos: 0 }),
        total_synced: data.total_synced,
        connected_extensions: data.connected_extensions,
        is_running: true,
        connection_status: ExtensionConnectionStatus::Connected as i32,
        error_message: String::new(),
    };
    
    NativeResponse {
        success: true,
        error: String::new(),
        request_id: request_id.to_string(),
        response: Some(native_response::Response::StatusResult(status)),
    }
}

fn handle_set_config(config: &ExtensionConfig, request_id: &str) -> NativeResponse {
    let domain_config = convert_proto_config(config);
    match usecase::native_host_sync::save_config(&domain_config) {
        Ok(_) => {
            let msg = "Config updated successfully".to_string();
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::ConfigResult(ConfigUpdateResult { message: msg })) }
        }
        Err(e) => {
            let err_msg = anyhow_chain_to_string(&e);
            let msg = format!("Failed to save config: {}", err_msg);
            NativeResponse { success: false, error: err_msg, request_id: request_id.to_string(), response: Some(native_response::Response::ConfigResult(ConfigUpdateResult { message: msg })) }
        }
    }
}

fn handle_health_check(_request: &HealthCheckRequest, request_id: &str) -> NativeResponse {
    let result = HealthCheckResult {
        message: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    NativeResponse {
        success: true,
        error: String::new(),
        request_id: request_id.to_string(),
        response: Some(native_response::Response::HealthCheckResult(result)),
    }
}

fn parse_message(message_bytes: &[u8]) -> HostResult<(NativeMessage, RequestFormat)> {
    // まずProtoBufとして直接パースを試みる
    if let Ok(message) = NativeMessage::decode(message_bytes) {
        log::info!("Parsed as raw ProtoBuf message");
        return Ok((message, RequestFormat::Protobuf));
    }

    // JSONとしてパース
    let json_str = std::str::from_utf8(message_bytes)?;

    // 1) Prost互換JSON（oneofはフィールド名で表現）
    if let Ok(message) = serde_json::from_str::<NativeMessage>(json_str) {
        log::info!("Parsed as JSON(prost) message");
        return Ok((message, RequestFormat::Json));
    }

    // 2) buf.build互換(case/value) 形式（構造体としてパース）
    #[derive(Debug, Deserialize, Default)]
    struct BufDmmGame {
        #[serde(default)] id: String,
        #[serde(default)] category: String,
        #[serde(default)] subcategory: String,
        #[serde(default)] title: String,
        #[serde(default, rename = "thumbnailUrl")] thumbnail_url: String,
        #[serde(default, rename = "imageUrl")] image_url: String,
    }
    #[derive(Debug, Deserialize, Default)]
    struct BufDlsiteGame {
        #[serde(default)] id: String,
        #[serde(default)] category: String,
        #[serde(default)] title: String,
        #[serde(default, rename = "thumbnailUrl")] thumbnail_url: String,
        #[serde(default, rename = "imageUrl")] image_url: String,
    }
    #[derive(Debug, Deserialize, Default)]
    struct BufExtensionConfig {
        #[serde(default, rename = "autoSync")] auto_sync: bool,
        #[serde(default, rename = "allowedDomains")] allowed_domains: Vec<String>,
        #[serde(default, rename = "syncIntervalMinutes")] sync_interval_minutes: u32,
        #[serde(default, rename = "debugMode")] debug_mode: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(tag = "case", content = "value")]
    enum BufCase {
        #[serde(rename = "syncDmmGames")] SyncDmmGames { #[serde(default)] games: Vec<BufDmmGame>, #[serde(default, rename = "extensionId")] extension_id: String },
        #[serde(rename = "syncDlsiteGames")] SyncDlsiteGames { #[serde(default)] games: Vec<BufDlsiteGame>, #[serde(default, rename = "extensionId")] extension_id: String },
        #[serde(rename = "getStatus")] GetStatus {},
        #[serde(rename = "setConfig")] SetConfig(#[serde(default)] BufExtensionConfig),
        #[serde(rename = "healthCheck")] HealthCheck {},
    }
    #[derive(Debug, Deserialize)]
    struct BufEnvelope { #[serde(default, rename = "requestId")] request_id: String, message: BufCase }

    let env: BufEnvelope = serde_json::from_str(json_str)?;

    let now = chrono::Utc::now().timestamp();
    let timestamp = Some(pbjson_types::Timestamp { seconds: now, nanos: 0 });

    let nm = match env.message {
        BufCase::SyncDmmGames { games, extension_id } => {
            let list = games.into_iter().map(|g| {
                let url = if !g.image_url.is_empty() { g.image_url } else { g.thumbnail_url };
                DmmGame { id: g.id, category: g.category, subcategory: g.subcategory, egs_info: None, title: g.title, image_url: url }
            }).collect();
            NativeMessage { timestamp, request_id: env.request_id.clone(), message: Some(native_message::Message::SyncDmmGames(DmmSyncGamesRequest { games: list, extension_id })) }
        }
        BufCase::SyncDlsiteGames { games, extension_id } => {
            let list = games.into_iter().map(|g| {
                let url = if !g.image_url.is_empty() { g.image_url } else { g.thumbnail_url };
                DlsiteGame { id: g.id, category: g.category, egs_info: None, title: g.title, image_url: url }
            }).collect();
            NativeMessage { timestamp, request_id: env.request_id.clone(), message: Some(native_message::Message::SyncDlsiteGames(DlsiteSyncGamesRequest { games: list, extension_id })) }
        }
        BufCase::GetStatus {} => {
            NativeMessage { timestamp, request_id: env.request_id.clone(), message: Some(native_message::Message::GetStatus(GetStatusRequest {})) }
        }
        BufCase::SetConfig(cfg) => {
            let cfg = ExtensionConfig { auto_sync: cfg.auto_sync, allowed_domains: cfg.allowed_domains, sync_interval_minutes: cfg.sync_interval_minutes, debug_mode: cfg.debug_mode };
            NativeMessage { timestamp, request_id: env.request_id.clone(), message: Some(native_message::Message::SetConfig(cfg)) }
        }
        BufCase::HealthCheck {} => {
            NativeMessage { timestamp, request_id: env.request_id.clone(), message: Some(native_message::Message::HealthCheck(HealthCheckRequest {})) }
        }
    };

    log::info!("Parsed as JSON(buf case/value) message");
    Ok((nm, RequestFormat::JsonBuf))
}

async fn send_response_with_format(response: &NativeResponse, format: RequestFormat) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = tokio_io::stdout();

    match format {
        RequestFormat::Protobuf => {
            // ProtoBuf形式で送信（生のバイナリデータ）
            let mut response_bytes = Vec::new();
            response.encode(&mut response_bytes)
                .map_err(|e| format!("Failed to encode protobuf response: {}", e))?;
            
            let length = response_bytes.len() as u32;
            
            stdout.write_all(&length.to_le_bytes()).await?;
            stdout.write_all(&response_bytes).await?;
            log::info!("Sent ProtoBuf response for request: {}", response.request_id);
        }
        RequestFormat::Json => {
            // JSON形式で送信
            let json_response = serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize JSON response: {}", e))?;
            
            let json_bytes = json_response.as_bytes();
            let length = json_bytes.len() as u32;
            
            stdout.write_all(&length.to_le_bytes()).await?;
            stdout.write_all(json_bytes).await?;
            log::info!("Sent JSON response for request: {}", response.request_id);
        }
        RequestFormat::JsonBuf => {
            // buf.buildのcase/value形式で送信
            let json_value = build_buf_json_response(response);
            let json_string = serde_json::to_string(&json_value)
                .map_err(|e| format!("Failed to serialize buf JSON: {}", e))?;
            let json_bytes = json_string.as_bytes();
            let length = json_bytes.len() as u32;
            stdout.write_all(&length.to_le_bytes()).await?;
            stdout.write_all(json_bytes).await?;
            log::info!("Sent Buf-JSON response for request: {}", response.request_id);
        }
    }
    
    stdout.flush().await?;
    Ok(())
}

fn build_buf_json_response(resp: &NativeResponse) -> serde_json::Value {
    use serde_json::json;
    let response = match &resp.response {
        Some(native_response::Response::SyncGamesResult(r)) => json!({
            "case": "syncGamesResult",
            "value": {
                "successCount": r.success_count,
                "errorCount": r.error_count,
                "errors": r.errors,
                "syncedGames": r.synced_games,
            }
        }),
        Some(native_response::Response::StatusResult(s)) => json!({
            "case": "statusResult",
            "value": {
                "lastSync": s.last_sync.as_ref().map(|t| json!({"seconds": t.seconds, "nanos": t.nanos})).unwrap_or(json!(null)),
                "totalSynced": s.total_synced,
                "connectedExtensions": s.connected_extensions,
                "isRunning": s.is_running,
                "connectionStatus": s.connection_status,
                "errorMessage": s.error_message,
            }
        }),
        Some(native_response::Response::ConfigResult(c)) => json!({
            "case": "configResult",
            "value": {"message": c.message}
        }),
        Some(native_response::Response::HealthCheckResult(h)) => json!({
            "case": "healthCheckResult",
            "value": {"message": h.message, "version": h.version}
        }),
        None => json!({"case": serde_json::Value::Null, "value": serde_json::Value::Null}),
    };

    json!({
        "success": resp.success,
        "error": resp.error,
        "requestId": resp.request_id,
        "response": response,
    })
}

// =============== 補助関数 ===============
// 補助: Proto -> Domain 変換（必要なら共通化）
fn convert_proto_config(cfg: &ExtensionConfig) -> crate::domain::extension::ExtensionConfig {
    crate::domain::extension::ExtensionConfig {
        auto_sync: cfg.auto_sync,
        allowed_domains: cfg.allowed_domains.clone(),
        sync_interval_minutes: cfg.sync_interval_minutes,
        debug_mode: cfg.debug_mode,
    }
}
