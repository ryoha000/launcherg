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
// interface層では直接sqlxなどのDBクライアントに依存しない
 

// プロトタイプを使用
use proto::generated::launcherg::{common::*, sync::*, status::*};
use infrastructure::repositoryimpl::{driver::Db as RepoDb, repository::Repositories};
use usecase::{native_host_sync::{NativeHostSyncUseCase, DmmSyncGameParam, DlsiteSyncGameParam}, native_host};

struct AppCtx {
    sync_usecase: NativeHostSyncUseCase<Repositories>,
}

#[derive(Debug)]
enum RequestFormat {
    Protobuf,
    Json,
    JsonBuf,
}

#[tokio::main]
async fn main() {
    // 標準エラー出力にログを記録
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    // UseCase/Repository を初期化
    let db_path = native_host::db_file_path();
    let repo_db = RepoDb::from_path(&db_path).await;
    let repositories = Repositories::new(repo_db);
    let sync_usecase = NativeHostSyncUseCase::new(Arc::new(repositories));
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
    let mut stdin = tokio_io::stdin();

    // メッセージ長を読み取り（4バイト、リトルエンディアン）
    let mut length_bytes = [0u8; 4];
    if let Err(e) = stdin.read_exact(&mut length_bytes).await {
        if e.kind() == ErrorKind::UnexpectedEof {
            // EOFの場合は正常終了
            return Ok(false);
        } else {
            return Err(e.into());
        }
    }
    
    let length = u32::from_le_bytes(length_bytes) as usize;
    
    // セキュリティチェック
    if length > 1024 * 1024 { // 1MB制限
        // サイズ超過はエラー応答を返して継続
        let error_msg = format!("Message too large: {} bytes (limit 1048576)", length);
        let response = NativeResponse {
            success: false,
            error: error_msg,
            request_id: String::new(),
            response: None,
        };
        // 形式は拡張が既定で使用するBuf JSONに合わせる
        send_response_with_format(&response, RequestFormat::JsonBuf).await?;
        return Ok(true);
    }
    
    // メッセージ本体を読み取り
    let mut message_bytes = vec![0u8; length];
    stdin.read_exact(&mut message_bytes).await?;
    
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

            let response = NativeResponse {
                success: false,
                error: err.to_string(),
                request_id,
                response: None,
            };
            send_response_with_format(&response, RequestFormat::JsonBuf).await?;
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
        .map(|g| DmmSyncGameParam { store_id: g.id.clone(), category: g.category.clone(), subcategory: g.subcategory.clone() })
        .collect();
    match ctx.sync_usecase.sync_dmm_games(params).await {
        Ok(success_count) => {
            native_host::bump_sync_counters(success_count);
            let result = SyncBatchResult { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::SyncGamesResult(result)) }
        }
        Err(e) => {
            // 例外チェーンのメッセージも含めて詳細を返す
            let mut msgs = vec![e.to_string()];
            let mut src = e.source();
            while let Some(s) = src {
                msgs.push(s.to_string());
                src = s.source();
            }
            let err_msg = msgs.join(": ");
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
        .map(|g| DlsiteSyncGameParam { store_id: g.id.clone(), category: g.category.clone() })
        .collect();
    match ctx.sync_usecase.sync_dlsite_games(params).await {
        Ok(success_count) => {
            native_host::bump_sync_counters(success_count);
            let result = SyncBatchResult { success_count, error_count: 0, errors: vec![], synced_games: input_ids };
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::SyncGamesResult(result)) }
        }
        Err(e) => {
            let mut msgs = vec![e.to_string()];
            let mut src = e.source();
            while let Some(s) = src {
                msgs.push(s.to_string());
                src = s.source();
            }
            let err_msg = msgs.join(": ");
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
    let data = native_host::get_status_data();
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
    let domain_config = crate::domain::extension::ExtensionConfig {
        auto_sync: config.auto_sync,
        allowed_domains: config.allowed_domains.clone(),
        sync_interval_minutes: config.sync_interval_minutes,
        debug_mode: config.debug_mode,
    };
    match native_host::save_config(&domain_config) {
        Ok(_) => {
            let msg = "Config updated successfully".to_string();
            NativeResponse { success: true, error: String::new(), request_id: request_id.to_string(), response: Some(native_response::Response::ConfigResult(ConfigUpdateResult { message: msg })) }
        }
        Err(e) => {
            let mut msgs = vec![e.to_string()];
            let mut src = e.source();
            while let Some(s) = src {
                msgs.push(s.to_string());
                src = s.source();
            }
            let err_msg = msgs.join(": ");
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

fn parse_message(message_bytes: &[u8]) -> Result<(NativeMessage, RequestFormat), Box<dyn std::error::Error>> {
    // まずProtoBufとして直接パースを試みる
    if let Ok(message) = NativeMessage::decode(message_bytes) {
        log::info!("Parsed as raw ProtoBuf message");
        return Ok((message, RequestFormat::Protobuf));
    }

    // JSONとしてパース
    let json_str = std::str::from_utf8(message_bytes)
        .map_err(|e| format!("Failed to parse message as UTF-8: {}", e))?;

    // 1) Prost互換JSON（oneofはフィールド名で表現）
    if let Ok(message) = serde_json::from_str::<NativeMessage>(json_str) {
        log::info!("Parsed as JSON(prost) message");
        return Ok((message, RequestFormat::Json));
    }

    // 2) buf.build互換(case/value) 形式
    let v: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON(Value): {}", e))?;
    let request_id = v.get("requestId").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let msg = v.get("message").and_then(|m| m.as_object()).ok_or("message not found")?;
    let case = msg.get("case").and_then(|c| c.as_str()).unwrap_or("");
    let value = msg.get("value").cloned().unwrap_or(serde_json::json!({}));

    let now = chrono::Utc::now().timestamp();
    let timestamp = Some(pbjson_types::Timestamp { seconds: now, nanos: 0 });

    let nm = match case {
        "syncDmmGames" => {
            let games = value.get("games").and_then(|g| g.as_array()).cloned().unwrap_or_default();
            let mut list = Vec::new();
            for g in games.into_iter() {
                let id = g.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let category = g.get("category").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let subcategory = g.get("subcategory").and_then(|s| s.as_str()).unwrap_or("").to_string();
                list.push(DmmGame { id, category, subcategory });
            }
            let extension_id = value.get("extensionId").and_then(|s| s.as_str()).unwrap_or("").to_string();
            NativeMessage { timestamp, request_id: request_id.clone(), message: Some(native_message::Message::SyncDmmGames(DmmSyncGamesRequest { games: list, extension_id })) }
        }
        "syncDlsiteGames" => {
            let games = value.get("games").and_then(|g| g.as_array()).cloned().unwrap_or_default();
            let mut list = Vec::new();
            for g in games.into_iter() {
                let id = g.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let category = g.get("category").and_then(|s| s.as_str()).unwrap_or("").to_string();
                list.push(DlsiteGame { id, category });
            }
            let extension_id = value.get("extensionId").and_then(|s| s.as_str()).unwrap_or("").to_string();
            NativeMessage { timestamp, request_id: request_id.clone(), message: Some(native_message::Message::SyncDlsiteGames(DlsiteSyncGamesRequest { games: list, extension_id })) }
        }
        "getStatus" => {
            NativeMessage { timestamp, request_id: request_id.clone(), message: Some(native_message::Message::GetStatus(GetStatusRequest {})) }
        }
        "setConfig" => {
            let auto_sync = value.get("autoSync").and_then(|b| b.as_bool()).unwrap_or(false);
            let debug_mode = value.get("debugMode").and_then(|b| b.as_bool()).unwrap_or(false);
            let sync_interval_minutes = value.get("syncIntervalMinutes").and_then(|n| n.as_u64()).unwrap_or(0) as u32;
            let allowed_domains = value.get("allowedDomains").and_then(|a| a.as_array()).map(|arr| arr.iter().filter_map(|s| s.as_str().map(|ss| ss.to_string())).collect()).unwrap_or_else(|| Vec::new());
            let cfg = ExtensionConfig { auto_sync, allowed_domains, sync_interval_minutes, debug_mode };
            NativeMessage { timestamp, request_id: request_id.clone(), message: Some(native_message::Message::SetConfig(cfg)) }
        }
        "healthCheck" => {
            NativeMessage { timestamp, request_id: request_id.clone(), message: Some(native_message::Message::HealthCheck(HealthCheckRequest {})) }
        }
        _ => return Err(format!("unsupported message case: {}", case).into()),
    };

    log::info!("Parsed as JSON(buf case/value) message: {}", case);
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

// 以降の低レベルSQL関数はUseCase/Repositoryに移譲したため存在しません