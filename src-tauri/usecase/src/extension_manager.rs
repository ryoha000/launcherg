use derive_new::new;
use chrono::Utc;

use super::error::UseCaseError;
use domain::{
    pubsub::{PubSubService, ExtensionConnectionPayload},
    extension::{SyncStatus, ExtensionConnectionStatus, ExtensionConfig, NativeMessagingHostClient, NativeMessagingHostClientFactory},
};
use std::sync::Arc;

#[derive(new)]
pub struct ExtensionManagerUseCase<P: PubSubService, F: NativeMessagingHostClientFactory> {
    pubsub: P,
    factory: Arc<F>,
}

impl<P: PubSubService, F: NativeMessagingHostClientFactory> ExtensionManagerUseCase<P, F> {
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

        // Native Messaging Hostクライアントを作成
        let native_messaging_client = match self.create_native_messaging_client() {
            Ok(client) => client,
            Err(e) => {
                let result_payload = ExtensionConnectionPayload {
                    connection_status: "host_not_found".to_string(),
                    is_running: false,
                    error_message: Some(e.to_string()),
                    timestamp: Utc::now(),
                };
                let _ = self.pubsub.notify("extension-connection-status", result_payload);
                
                return Ok(SyncStatus {
                    last_sync: None,
                    total_synced: 0,
                    connected_extensions: vec![],
                    is_running: false,
                    connection_status: ExtensionConnectionStatus::HostNotFound as i32,
                    error_message: e.to_string(),
                });
            }
        };

        // Native Messaging Hostとの通信を試行
        match native_messaging_client.health_check().await {
            Ok(health_ok) => {
                if health_ok {
                    // ヘルスチェック成功、ステータスを取得
                    match native_messaging_client.get_sync_status().await {
                        Ok(status) => {
                            let result_payload = ExtensionConnectionPayload {
                                connection_status: "connected".to_string(),
                                is_running: status.is_running,
                                error_message: None,
                                timestamp: Utc::now(),
                            };
                            let _ = self.pubsub.notify("extension-connection-status", result_payload);
                            Ok(status)
                        }
                        Err(e) => {
                            let result_payload = ExtensionConnectionPayload {
                                connection_status: "communication_error".to_string(),
                                is_running: false,
                                error_message: Some(e.to_string()),
                                timestamp: Utc::now(),
                            };
                            let _ = self.pubsub.notify("extension-connection-status", result_payload);
                            
                            Ok(SyncStatus {
                                last_sync: None,
                                total_synced: 0,
                                connected_extensions: vec![],
                                is_running: false,
                                connection_status: ExtensionConnectionStatus::CommunicationError as i32,
                                error_message: e.to_string(),
                            })
                        }
                    }
                } else {
                    let result_payload = ExtensionConnectionPayload {
                        connection_status: "health_check_failed".to_string(),
                        is_running: false,
                        error_message: Some("Health check failed".to_string()),
                        timestamp: Utc::now(),
                    };
                    let _ = self.pubsub.notify("extension-connection-status", result_payload);
                    
                    Ok(SyncStatus {
                        last_sync: None,
                        total_synced: 0,
                        connected_extensions: vec![],
                        is_running: false,
                        connection_status: ExtensionConnectionStatus::HealthCheckFailed as i32,
                        error_message: "Health check failed".to_string(),
                    })
                }
            }
            Err(e) => {
                let connection_status = match e.to_string().as_str() {
                    s if s.contains("not found") => "host_not_found",
                    s if s.contains("timeout") => "health_check_timeout",
                    s if s.contains("startup") => "host_startup_failed",
                    _ => "unknown_error",
                };
                
                let result_payload = ExtensionConnectionPayload {
                    connection_status: connection_status.to_string(),
                    is_running: false,
                    error_message: Some(e.to_string()),
                    timestamp: Utc::now(),
                };
                let _ = self.pubsub.notify("extension-connection-status", result_payload);

                Ok(SyncStatus {
                    last_sync: None,
                    total_synced: 0,
                    connected_extensions: vec![],
                    is_running: false,
                    connection_status: match connection_status {
                        "host_not_found" => ExtensionConnectionStatus::HostNotFound as i32,
                        "health_check_timeout" => ExtensionConnectionStatus::HealthCheckTimeout as i32,
                        "host_startup_failed" => ExtensionConnectionStatus::HostStartupFailed as i32,
                        _ => ExtensionConnectionStatus::UnknownError as i32,
                    },
                    error_message: e.to_string(),
                })
            }
        }
    }

    /// 拡張機能設定を更新
    pub async fn set_extension_config(&self, config: &ExtensionConfig) -> Result<String, UseCaseError> {
        let native_messaging_client = self.create_native_messaging_client()
            .map_err(|e| UseCaseError::NativeHostProcessError(e.to_string()))?;
            
        match native_messaging_client.set_config(config).await {
            Ok(message) => Ok(message),
            Err(e) => Err(UseCaseError::NativeHostProcessError(e.to_string())),
        }
    }

    /// Native Messaging Hostクライアントを作成
    fn create_native_messaging_client(&self) -> Result<F::Client, Box<dyn std::error::Error + Send + Sync>> {
        self.factory.create()
    }
}

