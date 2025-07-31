use std::{path::PathBuf, sync::Arc, time::Duration, process::Stdio};
use derive_new::new;
use serde_json::json;
use tokio::{process::{Child, Command}, time::timeout, io::{AsyncReadExt, AsyncWriteExt}};
use chrono::Utc;

use super::error::UseCaseError;
use crate::{
    domain::pubsub::{PubSubService, ExtensionConnectionPayload},
    infrastructure::repositoryimpl::repository::RepositoriesExt,
    native_messaging::protocol::{NativeMessage, NativeResponse, MessageType, SyncStatus, ExtensionConnectionStatus},
};

#[derive(new)]
pub struct ExtensionManagerUseCase<R: RepositoriesExt, P: PubSubService> {
    repositories: Arc<R>,
    pubsub: P,
    #[new(default)]
    native_host_path: Option<PathBuf>,
}

impl<R: RepositoriesExt, P: PubSubService> ExtensionManagerUseCase<R, P> {
    pub fn with_custom_path(repositories: Arc<R>, pubsub: P, native_host_path: PathBuf) -> Self {
        Self {
            repositories,
            pubsub,
            native_host_path: Some(native_host_path),
        }
    }
}

impl<R: RepositoriesExt, P: PubSubService> ExtensionManagerUseCase<R, P> {
    /// ブラウザ拡張機能の接続状況をチェックする
    pub async fn check_extension_connection(&self) -> Result<SyncStatus, UseCaseError> {
        // 接続開始をPubSubで通知
        let connecting_payload = ExtensionConnectionPayload {
            connection_status: "connecting".to_string(),
            is_running: false,
            error_message: None,
            timestamp: Utc::now(),
        };
        let _ = self.pubsub.notify("extension-connection-status", connecting_payload);

        let (connection_status, error_message) = self.test_native_host_communication_detailed().await;
        
        let is_running = matches!(connection_status, ExtensionConnectionStatus::Connected);
        
        // 接続結果をPubSubで通知
        let result_payload = ExtensionConnectionPayload {
            connection_status: serde_json::to_string(&connection_status)
                .unwrap_or_else(|_| "unknown_error".to_string())
                .trim_matches('"').to_string(),
            is_running,
            error_message: error_message.clone(),
            timestamp: Utc::now(),
        };
        let _ = self.pubsub.notify("extension-connection-status", result_payload);

        Ok(SyncStatus {
            last_sync: if is_running { Some(Utc::now()) } else { None },
            total_synced: 0, // TODO: 実際の値を取得
            connected_extensions: if is_running { 
                vec!["extension-detected".to_string()] 
            } else { 
                vec![] 
            },
            is_running,
            connection_status,
            error_message,
        })
    }

    /// Native Messaging Hostプロセスとの通信をテストし、詳細な状態を返す
    async fn test_native_host_communication_detailed(&self) -> (ExtensionConnectionStatus, Option<String>) {
        // 1. プロセスを起動
        let mut process = match self.spawn_native_host_process().await {
            Ok(process) => process,
            Err(e) => {
                return match e {
                    UseCaseError::NativeHostProcessError(msg) if msg.contains("not found") || msg.contains("見つかりません") => {
                        (ExtensionConnectionStatus::HostNotFound, Some(msg))
                    }
                    UseCaseError::NativeHostProcessError(msg) => {
                        (ExtensionConnectionStatus::HostStartupFailed, Some(msg))
                    }
                    _ => (ExtensionConnectionStatus::UnknownError, Some(e.to_string())),
                }
            }
        };

        // 2. ヘルスチェックメッセージを送信し、応答を待機
        let health_check_result = self.send_health_check(&mut process).await;

        // 3. プロセスを確実に終了
        let termination_result = Self::ensure_process_terminated(process).await;

        // 4. 結果を解析
        match health_check_result {
            Ok(true) => {
                // ヘルスチェック成功、プロセス終了の結果を確認
                match termination_result {
                    Ok(_) => (ExtensionConnectionStatus::Connected, None),
                    Err(e) => (
                        ExtensionConnectionStatus::ProcessTerminationError, 
                        Some(format!("ヘルスチェック成功、但しプロセス終了エラー: {}", e))
                    ),
                }
            }
            Ok(false) => (
                ExtensionConnectionStatus::HealthCheckFailed,
                Some("Native Messaging Hostからエラーレスポンスを受信".to_string())
            ),
            Err(e) => {
                let error_msg = e.to_string();
                let status = if error_msg.contains("タイムアウト") || error_msg.contains("timeout") {
                    ExtensionConnectionStatus::HealthCheckTimeout
                } else if error_msg.contains("パース") || error_msg.contains("parse") || error_msg.contains("JSON") {
                    ExtensionConnectionStatus::CommunicationError
                } else if error_msg.contains("stdin") || error_msg.contains("stdout") || error_msg.contains("通信") {
                    ExtensionConnectionStatus::CommunicationError
                } else {
                    ExtensionConnectionStatus::UnknownError
                };
                (status, Some(error_msg))
            }
        }
    }

    /// Native Messaging Hostプロセスとの通信をテストする（後方互換性用）
    async fn test_native_host_communication(&self) -> Result<bool, UseCaseError> {
        let (status, _) = self.test_native_host_communication_detailed().await;
        Ok(matches!(status, ExtensionConnectionStatus::Connected))
    }

    /// Native Messaging Hostプロセスを子プロセスとして起動
    async fn spawn_native_host_process(&self) -> Result<Child, UseCaseError> {
        let host_path = match &self.native_host_path {
            Some(path) => path.clone(),
            None => {
                // デフォルトパスを使用
                let exe_dir = std::env::current_exe()
                    .map_err(|e| UseCaseError::NativeHostProcessError(
                        format!("実行ファイルのパスを取得できません: {}", e)
                    ))?
                    .parent()
                    .ok_or_else(|| UseCaseError::NativeHostProcessError(
                        "実行ファイルのディレクトリが見つかりません".to_string()
                    ))?
                    .to_path_buf();
                
                exe_dir.join("native-messaging-host.exe")
            }
        };

        let child = Command::new(&host_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| UseCaseError::NativeHostProcessError(
                format!("Native Messaging Hostプロセスの起動に失敗: {} (path: {})", e, host_path.display())
            ))?;

        Ok(child)
    }

    /// ヘルスチェックメッセージを送信し、応答を待機
    async fn send_health_check(&self, process: &mut Child) -> Result<bool, UseCaseError> {
        let stdin = process.stdin.as_mut()
            .ok_or_else(|| UseCaseError::NativeHostProcessError(
                "プロセスのstdinが利用できません".to_string()
            ))?;
        
        let stdout = process.stdout.as_mut()
            .ok_or_else(|| UseCaseError::NativeHostProcessError(
                "プロセスのstdoutが利用できません".to_string()
            ))?;

        // ヘルスチェックメッセージを作成
        let health_check_message = NativeMessage::new(
            MessageType::HealthCheck,
            json!({}),
        );

        let message_json = serde_json::to_string(&health_check_message)
            .map_err(|e| UseCaseError::NativeHostProcessError(
                format!("メッセージのシリアライズに失敗: {}", e)
            ))?;

        let message_bytes = message_json.as_bytes();
        let message_length = message_bytes.len() as u32;

        // Native Messaging Protocolに従ってメッセージを送信
        // 1. メッセージ長（4バイト、リトルエンディアン）
        timeout(Duration::from_secs(5), stdin.write_all(&message_length.to_le_bytes())).await
            .map_err(|_| UseCaseError::NativeHostProcessError("メッセージ長送信タイムアウト".to_string()))?
            .map_err(|e| UseCaseError::NativeHostProcessError(
                format!("メッセージ長の送信に失敗: {}", e)
            ))?;

        // 2. メッセージ本体
        timeout(Duration::from_secs(5), stdin.write_all(message_bytes)).await
            .map_err(|_| UseCaseError::NativeHostProcessError("メッセージ送信タイムアウト".to_string()))?
            .map_err(|e| UseCaseError::NativeHostProcessError(
                format!("メッセージの送信に失敗: {}", e)
            ))?;

        timeout(Duration::from_secs(5), stdin.flush()).await
            .map_err(|_| UseCaseError::NativeHostProcessError("フラッシュタイムアウト".to_string()))?
            .map_err(|e| UseCaseError::NativeHostProcessError(
                format!("フラッシュに失敗: {}", e)
            ))?;

        // 応答を読み取り
        let response_result = timeout(Duration::from_secs(30), async {
            // 1. 応答長を読み取り
            let mut length_bytes = [0u8; 4];
            stdout.read_exact(&mut length_bytes).await?;
            let response_length = u32::from_le_bytes(length_bytes) as usize;

            // セキュリティチェック
            if response_length > 1024 * 1024 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "応答サイズが大きすぎます"
                ));
            }

            // 2. 応答本体を読み取り
            let mut response_bytes = vec![0u8; response_length];
            stdout.read_exact(&mut response_bytes).await?;

            Ok(response_bytes)
        }).await;

        match response_result {
            Ok(Ok(response_bytes)) => {
                // JSONパース
                let response_str = String::from_utf8(response_bytes)
                    .map_err(|e| UseCaseError::NativeHostProcessError(
                        format!("レスポンスのUTF-8変換に失敗: {}", e)
                    ))?;

                let response: NativeResponse = serde_json::from_str(&response_str)
                    .map_err(|e| UseCaseError::NativeHostProcessError(
                        format!("レスポンスのパースに失敗: {}", e)
                    ))?;

                Ok(response.success)
            }
            Ok(Err(e)) => Err(UseCaseError::NativeHostProcessError(
                format!("応答の読み取りに失敗: {}", e)
            )),
            Err(_) => Err(UseCaseError::NativeHostProcessError(
                "応答待機タイムアウト".to_string()
            )),
        }
    }

    /// プロセスを確実に終了させる
    async fn ensure_process_terminated(mut process: Child) -> Result<(), UseCaseError> {
        // まず正常終了を試行
        if let Some(stdin) = process.stdin.take() {
            drop(stdin); // stdinを閉じることで、プロセスに終了を促す
        }

        // プロセスの終了を5秒待機
        let wait_result = timeout(Duration::from_secs(5), process.wait()).await;
        
        match wait_result {
            Ok(Ok(_)) => {
                // 正常終了
                Ok(())
            }
            Ok(Err(e)) => Err(UseCaseError::NativeHostProcessError(
                format!("プロセス終了の待機に失敗: {}", e)
            )),
            Err(_) => {
                // タイムアウト: 強制終了を試行
                match process.kill().await {
                    Ok(_) => {
                        // killが成功したら、wait()を再試行
                        let _ = process.wait().await;
                        Ok(())
                    }
                    Err(e) => Err(UseCaseError::NativeHostProcessError(
                        format!("プロセスの強制終了に失敗: {}", e)
                    )),
                }
            }
        }
    }
}

