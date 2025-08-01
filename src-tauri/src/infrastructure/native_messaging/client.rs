use std::{path::PathBuf, process::{Command, Stdio}, io::Write, time::Duration};
use tokio::time::timeout;
use prost::Message;
use pbjson_types::Timestamp;
use chrono::Utc;

use crate::{
    domain::extension::{NativeMessagingHostClient, SyncStatus, ExtensionConfig},
    usecase::extension_installer::ExtensionInstallerUseCase,
};
use super::proto::generated::launcherg::{common::*, status};

pub struct NativeMessagingHostClientImpl {
    native_host_path: Option<PathBuf>,
    app_handle: Option<std::sync::Arc<tauri::AppHandle>>,
}

impl NativeMessagingHostClientImpl {
    pub fn new() -> Self {
        Self {
            native_host_path: None,
            app_handle: None,
        }
    }

    pub fn with_custom_path(native_host_path: PathBuf) -> Self {
        Self {
            native_host_path: Some(native_host_path),
            app_handle: None,
        }
    }

    pub fn with_app_handle(app_handle: std::sync::Arc<tauri::AppHandle>) -> Self {
        Self {
            native_host_path: None,
            app_handle: Some(app_handle),
        }
    }

    /// Native Messaging Hostの実行ファイルパスを取得
    fn get_native_host_path(&self) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(path) = &self.native_host_path {
            return Ok(path.clone());
        }

        // AppHandleがある場合はレジストリからパスを取得
        if let Some(app_handle) = &self.app_handle {
            match self.get_native_host_path_from_registry(app_handle) {
                Ok(path) => return Ok(path),
                Err(e) => {
                    // レジストリから取得できない場合はフォールバック
                    log::warn!("Failed to get path from registry: {}", e);
                }
            }
        }

        // フォールバック: 現在の作業ディレクトリを基準にした絶対パスを構築
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        // デフォルトパスを試行（絶対パスで）
        let possible_paths = vec![
            current_dir.join("native-messaging-host.exe"),
            current_dir.join("src-tauri").join("native-messaging-host.exe"),
            current_dir.join("src-tauri").join("target").join("release").join("native-messaging-host.exe"),
            current_dir.join("src-tauri").join("target").join("debug").join("native-messaging-host.exe"),
        ];

        for path in &possible_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // デバッグ用にパス情報を含むエラーメッセージを作成
        let path_info = possible_paths
            .iter()
            .map(|p| format!("- {}: {}", p.display(), if p.exists() { "EXISTS" } else { "NOT FOUND" }))
            .collect::<Vec<_>>()
            .join("\n");

        Err(format!(
            "Native Messaging Host executable not found. Searched paths:\n{}", 
            path_info
        ).into())
    }

    /// レジストリからNative Messaging Hostのパスを取得
    fn get_native_host_path_from_registry(&self, app_handle: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let installer = ExtensionInstallerUseCase::new(std::sync::Arc::new(app_handle.clone()));
        let registry_keys = installer.check_registry_keys()
            .map_err(|e| format!("Failed to check registry: {}", e))?;

        // 登録されているレジストリキーからマニフェストファイルのパスを探す
        for key_info in registry_keys {
            if key_info.exists {
                if let Some(manifest_path) = key_info.value {
                    // マニフェストファイルから実行ファイルのパスを読み取り
                    match self.get_executable_path_from_manifest(&manifest_path) {
                        Ok(exe_path) => return Ok(exe_path),
                        Err(e) => {
                            log::warn!("Failed to get executable path from manifest {}: {}", manifest_path, e);
                            continue;
                        }
                    }
                }
            }
        }

        Err("No valid registry entry found".into())
    }

    /// マニフェストファイルから実行ファイルのパスを読み取り
    fn get_executable_path_from_manifest(&self, manifest_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let manifest_content = std::fs::read_to_string(manifest_path)
            .map_err(|e| format!("Failed to read manifest file {}: {}", manifest_path, e))?;

        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;

        let exe_path = manifest.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| "No 'path' field in manifest")?;

        let exe_path = PathBuf::from(exe_path);
        if !exe_path.exists() {
            return Err(format!("Executable not found: {}", exe_path.display()).into());
        }

        Ok(exe_path)
    }

    /// Native Messaging Hostプロセスにメッセージを送信
    async fn send_message_to_native_host(&self, message: &NativeMessage) -> Result<NativeResponse, Box<dyn std::error::Error + Send + Sync>> {
        let native_host_path = self.get_native_host_path()?;
        
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

#[async_trait::async_trait]
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