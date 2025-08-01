mod proto;

use std::io::{self, Read, Write};
use prost::Message;
use pbjson_types::Timestamp;
use chrono::Utc;

// プロトタイプを使用
use proto::generated::launcherg::{common::*, sync::*, status::*};

fn main() {
    // 標準エラー出力にログを記録
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    log::info!("Native Messaging Host started");

    loop {
        match handle_message() {
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

fn handle_message() -> Result<bool, Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    // メッセージ長を読み取り（4バイト、リトルエンディアン）
    let mut length_bytes = [0u8; 4];
    match handle.read_exact(&mut length_bytes) {
        Ok(_) => {},
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // EOFの場合は正常終了
            return Ok(false);
        }
        Err(e) => return Err(e.into()),
    }
    
    let length = u32::from_le_bytes(length_bytes) as usize;
    
    // セキュリティチェック
    if length > 1024 * 1024 { // 1MB制限
        return Err("Message too large".into());
    }
    
    // メッセージ本体を読み取り
    let mut message_bytes = vec![0u8; length];
    handle.read_exact(&mut message_bytes)?;
    
    let message = NativeMessage::decode(&message_bytes[..])
        .map_err(|e| format!("Failed to decode protobuf message: {}", e))?;
    
    let message_type = match &message.message {
        Some(native_message::Message::SyncGames(_)) => "sync_games",
        Some(native_message::Message::GetStatus(_)) => "get_status",
        Some(native_message::Message::SetConfig(_)) => "set_config",
        Some(native_message::Message::HealthCheck(_)) => "health_check",
        None => "unknown",
    };
    log::info!("Received message type: {}", message_type);
    
    // メッセージタイプに応じて処理
    let response = match &message.message {
        Some(native_message::Message::SyncGames(req)) => handle_sync_games(req, &message.request_id),
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
    
    // レスポンスを送信
    send_response(&response)?;
    
    Ok(true)
}

fn handle_sync_games(request: &SyncGamesRequest, request_id: &str) -> NativeResponse {
    log::info!("Syncing {} games from store: {}", request.games.len(), request.store);
    
    let result = SyncBatchResult {
        success_count: 3,
        error_count: 1,
        errors: vec!["Game 'Test Game 4' not found in ErogameScape database".to_string()],
        synced_games: vec!["Test Game 1".to_string(), "Test Game 2".to_string(), "Test Game 3".to_string()],
    };
    
    NativeResponse {
        success: true,
        error: String::new(),
        request_id: request_id.to_string(),
        response: Some(native_response::Response::SyncGamesResult(result)),
    }
}

fn handle_get_status(_request: &GetStatusRequest, request_id: &str) -> NativeResponse {
    let status = SyncStatus {
        last_sync: Some(Timestamp {
            seconds: Utc::now().timestamp(),
            nanos: 0,
        }),
        total_synced: 42,
        connected_extensions: vec!["chrome-extension://abcdefghijklmnop".to_string()],
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
    log::info!("Config updated: auto_sync={}, debug_mode={}", config.auto_sync, config.debug_mode);
    
    let result = ConfigUpdateResult {
        message: "Config updated successfully".to_string(),
    };
    
    NativeResponse {
        success: true,
        error: String::new(),
        request_id: request_id.to_string(),
        response: Some(native_response::Response::ConfigResult(result)),
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

fn send_response(response: &NativeResponse) -> Result<(), Box<dyn std::error::Error>> {
    let mut response_bytes = Vec::new();
    response.encode(&mut response_bytes)
        .map_err(|e| format!("Failed to encode protobuf response: {}", e))?;
    
    let length = response_bytes.len() as u32;
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    // メッセージ長を送信（4バイト、リトルエンディアン）
    handle.write_all(&length.to_le_bytes())?;
    // メッセージ本体を送信
    handle.write_all(&response_bytes)?;
    handle.flush()?;
    
    log::info!("Sent response for request: {}", response.request_id);
    
    Ok(())
}