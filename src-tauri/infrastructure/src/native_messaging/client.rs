use std::{path::PathBuf, process::{Command, Stdio}, io::Write, time::Duration};
use tokio::time::timeout;
use prost::Message;
use pbjson_types::Timestamp;
use chrono::Utc;

use domain::extension::{NativeMessagingHostClient, SyncStatus, ExtensionConfig};
use super::proto::generated::launcherg::{common::*, status};

pub struct NativeMessagingHostClientImpl {
    native_host_path: PathBuf,
}

impl NativeMessagingHostClientImpl {
    pub fn new(native_host_path: PathBuf) -> Self {
        Self {
            native_host_path,
        }
    }

    /// Native Messaging Hostの実行ファイルパスを取得
    fn get_native_host_path(&self) -> &PathBuf {
        &self.native_host_path
    }


    /// Native Messaging Hostプロセスにメッセージを送信
    async fn send_message_to_native_host(&self, message: &NativeMessage) -> Result<NativeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let native_host_path = self.get_native_host_path();
        
        let mut child = Command::new(native_host_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Native Host: {}", e))?;

        let mut stdin = child.stdin.take().ok_or_else(|| {
            "Failed to get stdin"
        })?;

        // ProtoBufメッセージをエンコード
        let mut message_bytes = Vec::new();
        message.encode(&mut message_bytes)
            .map_err(|e| format!("Failed to encode protobuf message: {}", e))?;

        // Native Messagingプロトコル：長さ（4バイト）+ バイナリデータ
        let length = (message_bytes.len() as u32).to_le_bytes();
        
        stdin.write_all(&length).map_err(|e| {
            format!("Failed to write length: {}", e)
        })?;
        stdin.write_all(&message_bytes).map_err(|e| {
            format!("Failed to write message: {}", e)
        })?;
        drop(stdin);

        // レスポンスを読み取り
        let output = child.wait_with_output().map_err(|e| {
            format!("Failed to read output: {}", e)
        })?;

        if !output.status.success() {
            return Err(format!(
                "Native Host process failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        let stdout = output.stdout;
        if stdout.len() < 4 {
            return Err("Invalid response format".into());
        }

        // 長さを読み取り
        let response_length = u32::from_le_bytes([stdout[0], stdout[1], stdout[2], stdout[3]]) as usize;
        
        if stdout.len() < 4 + response_length {
            return Err("Incomplete response".into());
        }

        // ProtoBufレスポンスをデコード
        let response = NativeResponse::decode(&stdout[4..4 + response_length])
            .map_err(|e| format!("Failed to decode protobuf response: {}", e))?;
        
        Ok(response)
    }

    /// ProtoBuf SyncStatusをドメインモデルに変換
    fn convert_sync_status(&self, proto_status: &status::SyncStatus) -> SyncStatus {
        SyncStatus {
            last_sync: proto_status.last_sync.clone(),
            total_synced: proto_status.total_synced,
            connected_extensions: proto_status.connected_extensions.clone(),
            is_running: proto_status.is_running,
            connection_status: proto_status.connection_status,
            error_message: proto_status.error_message.clone(),
        }
    }

    /// ドメインモデルをProtoBuf ExtensionConfigに変換
    fn convert_extension_config(&self, config: &ExtensionConfig) -> status::ExtensionConfig {
        status::ExtensionConfig {
            auto_sync: config.auto_sync,
            allowed_domains: config.allowed_domains.clone(),
            sync_interval_minutes: config.sync_interval_minutes,
            debug_mode: config.debug_mode,
        }
    }
}

impl NativeMessagingHostClient for NativeMessagingHostClientImpl {
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let message = NativeMessage {
            timestamp: Some(Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
            request_id: "health-check".to_string(),
            message: Some(native_message::Message::HealthCheck(HealthCheckRequest {})),
        };
        
        match timeout(Duration::from_secs(5), self.send_message_to_native_host(&message)).await {
            Ok(Ok(response)) => {
                Ok(response.success)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Health check timeout".into()),
        }
    }

    async fn get_sync_status(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let message = NativeMessage {
            timestamp: Some(Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
            request_id: "get-status".to_string(),
            message: Some(native_message::Message::GetStatus(GetStatusRequest {})),
        };
        
        match timeout(Duration::from_secs(5), self.send_message_to_native_host(&message)).await {
            Ok(Ok(response)) => {
                if response.success {
                    if let Some(native_response::Response::StatusResult(status)) = response.response {
                        Ok(self.convert_sync_status(&status))
                    } else {
                        Err("Invalid response format: expected status result".into())
                    }
                } else {
                    Err(format!("Failed to get status: {}", response.error).into())
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Get status timeout".into()),
        }
    }

    async fn set_config(&self, config: &ExtensionConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let proto_config = self.convert_extension_config(config);
        let message = NativeMessage {
            timestamp: Some(Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: 0,
            }),
            request_id: "set-config".to_string(),
            message: Some(native_message::Message::SetConfig(proto_config)),
        };
        
        match timeout(Duration::from_secs(5), self.send_message_to_native_host(&message)).await {
            Ok(Ok(response)) => {
                if response.success {
                    if let Some(native_response::Response::ConfigResult(result)) = response.response {
                        Ok(result.message)
                    } else {
                        Ok("Config updated successfully".to_string())
                    }
                } else {
                    Err(format!("Failed to set config: {}", response.error).into())
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Set config timeout".into()),
        }
    }
}