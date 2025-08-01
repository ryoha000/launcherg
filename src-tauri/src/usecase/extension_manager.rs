use std::{path::PathBuf, sync::Arc};
use derive_new::new;
use chrono::Utc;

use super::error::UseCaseError;
use crate::{
    domain::pubsub::{PubSubService, ExtensionConnectionPayload}, 
    infrastructure::repositoryimpl::repository::RepositoriesExt,
    interface::models::extension::{SyncStatus, ExtensionConnectionStatus},
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

        // TODO: Native Messaging Host通信の実装を簡素化
        let connection_status = ExtensionConnectionStatus::HostNotFound;
        let error_message = Some("Native Messaging Host機能は現在利用できません".to_string());
        
        let is_running = false;
        
        // 接続結果をPubSubで通知
        let result_payload = ExtensionConnectionPayload {
            connection_status: "host_not_found".to_string(),
            is_running,
            error_message: error_message.clone(),
            timestamp: Utc::now(),
        };
        let _ = self.pubsub.notify("extension-connection-status", result_payload);

        Ok(SyncStatus {
            last_sync: None,
            total_synced: 0,
            connected_extensions: vec![],
            is_running,
            connection_status: connection_status as i32,
            error_message: error_message.unwrap_or_else(String::new),
        })
    }
}

