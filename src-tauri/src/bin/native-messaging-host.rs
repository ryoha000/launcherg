use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use serde_json;

// プロトコル定義の簡易版（実際はsrc/native_messaging/protocol.rsを使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeMessage {
    #[serde(rename = "type")]
    type_: String,
    payload: serde_json::Value,
    timestamp: String,
    request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
    request_id: String,
}

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
    
    let message_str = String::from_utf8(message_bytes)?;
    let message: NativeMessage = serde_json::from_str(&message_str)?;
    
    log::info!("Received message type: {}", message.type_);
    
    // メッセージタイプに応じて処理
    let response = match message.type_.as_str() {
        "sync_games" => handle_sync_games(&message),
        "get_status" => handle_get_status(&message),
        "set_config" => handle_set_config(&message),
        "health_check" => handle_health_check(&message),
        _ => NativeResponse {
            success: false,
            data: None,
            error: Some(format!("Unknown message type: {}", message.type_)),
            request_id: message.request_id,
        },
    };
    
    // レスポンスを送信
    send_response(&response)?;
    
    Ok(true)
}

fn handle_sync_games(message: &NativeMessage) -> NativeResponse {
    // 固定値を返す実装
    let result = serde_json::json!({
        "success_count": 3,
        "error_count": 1,
        "errors": ["Game 'Test Game 4' not found in ErogameScape database"],
        "synced_games": ["Test Game 1", "Test Game 2", "Test Game 3"]
    });
    
    NativeResponse {
        success: true,
        data: Some(result),
        error: None,
        request_id: message.request_id.clone(),
    }
}

fn handle_get_status(message: &NativeMessage) -> NativeResponse {
    // 固定値を返す実装
    let status = serde_json::json!({
        "last_sync": "2025-01-30T12:34:56Z",
        "total_synced": 42,
        "connected_extensions": ["chrome-extension://abcdefghijklmnop"],
        "is_running": true
    });
    
    NativeResponse {
        success: true,
        data: Some(status),
        error: None,
        request_id: message.request_id.clone(),
    }
}

fn handle_set_config(message: &NativeMessage) -> NativeResponse {
    // 設定を受け取ったことにして成功を返す
    log::info!("Config updated: {:?}", message.payload);
    
    NativeResponse {
        success: true,
        data: Some(serde_json::json!("Config updated successfully")),
        error: None,
        request_id: message.request_id.clone(),
    }
}

fn handle_health_check(message: &NativeMessage) -> NativeResponse {
    NativeResponse {
        success: true,
        data: Some(serde_json::json!("OK")),
        error: None,
        request_id: message.request_id.clone(),
    }
}

fn send_response(response: &NativeResponse) -> Result<(), Box<dyn std::error::Error>> {
    let response_str = serde_json::to_string(response)?;
    let response_bytes = response_str.as_bytes();
    let length = response_bytes.len() as u32;
    
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    // メッセージ長を送信（4バイト、リトルエンディアン）
    handle.write_all(&length.to_le_bytes())?;
    // メッセージ本体を送信
    handle.write_all(response_bytes)?;
    handle.flush()?;
    
    log::info!("Sent response for request: {}", response.request_id);
    
    Ok(())
}