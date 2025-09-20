use serde_json::json;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::time::timeout;

use domain::extension::{ExtensionConfig, NativeMessagingHostClient, SyncStatus};

pub struct NativeMessagingHostClientImpl {
    native_host_path: PathBuf,
}

impl NativeMessagingHostClientImpl {
    pub fn new(native_host_path: PathBuf) -> Self {
        Self { native_host_path }
    }

    /// Native Messaging Hostの実行ファイルパスを取得
    fn get_native_host_path(&self) -> &PathBuf {
        &self.native_host_path
    }

    /// Native Messaging Hostプロセスにメッセージを送信
    async fn send_message_to_native_host_json(
        &self,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let native_host_path = self.get_native_host_path();

        let mut child = Command::new(native_host_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Native Host: {}", e))?;

        let mut stdin = child.stdin.take().ok_or_else(|| "Failed to get stdin")?;

        // JSON(Buf)メッセージをエンコード
        let message_string = serde_json::to_string(payload)?;
        let message_bytes = message_string.as_bytes();

        // Native Messagingプロトコル：長さ（4バイト）+ バイナリデータ
        let length = (message_bytes.len() as u32).to_le_bytes();

        stdin
            .write_all(&length)
            .map_err(|e| format!("Failed to write length: {}", e))?;
        stdin
            .write_all(message_bytes)
            .map_err(|e| format!("Failed to write message: {}", e))?;
        drop(stdin);

        // レスポンスを読み取り
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to read output: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Native Host process failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = output.stdout;
        if stdout.len() < 4 {
            return Err("Invalid response format".into());
        }

        // 長さを読み取り
        let response_length =
            u32::from_le_bytes([stdout[0], stdout[1], stdout[2], stdout[3]]) as usize;

        if stdout.len() < 4 + response_length {
            return Err("Incomplete response".into());
        }

        // JSON(Buf) レスポンスをデコード
        let value: serde_json::Value = serde_json::from_slice(&stdout[4..4 + response_length])?;
        Ok(value)
    }

    /// SyncStatusをドメインモデルに変換（Buf JSON）
    fn convert_sync_status(&self, v: &serde_json::Value) -> SyncStatus {
        let last_sync = None;
        SyncStatus {
            last_sync,
            total_synced: v.get("totalSynced").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
            connected_extensions: v
                .get("connectedExtensions")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            is_running: v
                .get("isRunning")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            connection_status: v
                .get("connectionStatus")
                .and_then(|x| x.as_i64())
                .unwrap_or_default() as i32,
            error_message: v
                .get("errorMessage")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    /// ドメインモデルを Buf JSON の設定に変換
    fn convert_extension_config_json(&self, config: &ExtensionConfig) -> serde_json::Value {
        json!({
            "autoSync": config.auto_sync,
            "allowedDomains": config.allowed_domains,
            "syncIntervalMinutes": config.sync_interval_minutes,
            "debugMode": config.debug_mode,
        })
    }
}

impl NativeMessagingHostClient for NativeMessagingHostClientImpl {
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let payload = json!({
            "requestId": "health-check",
            "message": { "case": "healthCheck", "value": {} },
        });

        match timeout(
            Duration::from_secs(5),
            self.send_message_to_native_host_json(&payload),
        )
        .await
        {
            Ok(Ok(response)) => Ok(response
                .get("success")
                .and_then(|x| x.as_bool())
                .unwrap_or(false)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Health check timeout".into()),
        }
    }

    async fn get_sync_status(
        &self,
    ) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let payload = json!({
            "requestId": "get-status",
            "message": { "case": "getStatus", "value": {} },
        });

        match timeout(
            Duration::from_secs(5),
            self.send_message_to_native_host_json(&payload),
        )
        .await
        {
            Ok(Ok(response)) => {
                if response
                    .get("success")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false)
                {
                    let status = response
                        .get("response")
                        .and_then(|r| r.get("value"))
                        .and_then(|v| v.get("status"));
                    let status = status.unwrap_or(&serde_json::Value::Null);
                    Ok(self.convert_sync_status(status))
                } else {
                    let err = response.get("error").and_then(|x| x.as_str()).unwrap_or("");
                    Err(format!("Failed to get status: {}", err).into())
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Get status timeout".into()),
        }
    }

    async fn set_config(
        &self,
        config: &ExtensionConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = self.convert_extension_config_json(config);
        let payload = json!({
            "requestId": "set-config",
            "message": { "case": "setConfig", "value": cfg },
        });

        match timeout(
            Duration::from_secs(5),
            self.send_message_to_native_host_json(&payload),
        )
        .await
        {
            Ok(Ok(response)) => {
                if response
                    .get("success")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false)
                {
                    let msg = response
                        .get("response")
                        .and_then(|r| r.get("value"))
                        .and_then(|v| v.get("message"))
                        .and_then(|x| x.as_str())
                        .unwrap_or("Config updated successfully");
                    Ok(msg.to_string())
                } else {
                    let err = response.get("error").and_then(|x| x.as_str()).unwrap_or("");
                    Err(format!("Failed to set config: {}", err).into())
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Set config timeout".into()),
        }
    }
}
