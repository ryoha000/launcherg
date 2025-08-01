use async_trait::async_trait;
use crate::domain::extension::{NativeMessagingHostClient, SyncStatus, ExtensionConfig, ExtensionConnectionStatus};

pub struct MockNativeMessagingHostClient {
    should_succeed: bool,
    health_check_result: bool,
    path_exists: bool,
}

impl Default for MockNativeMessagingHostClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockNativeMessagingHostClient {
    pub fn new() -> Self {
        Self {
            should_succeed: true,
            health_check_result: true,
            path_exists: true,
        }
    }

    pub fn with_failure() -> Self {
        Self {
            should_succeed: false,
            health_check_result: false,
            path_exists: false,
        }
    }

    pub fn with_path_not_exists() -> Self {
        Self {
            should_succeed: false,
            health_check_result: false,
            path_exists: false,
        }
    }

    pub fn with_health_check_failure() -> Self {
        Self {
            should_succeed: false,
            health_check_result: false,
            path_exists: true,
        }
    }
}

#[async_trait]
impl NativeMessagingHostClient for MockNativeMessagingHostClient {
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if !self.path_exists {
            return Err("Native Messaging Host executable not found".into());
        }
        
        if !self.should_succeed {
            return Err("Health check timeout".into());
        }
        
        Ok(self.health_check_result)
    }

    async fn get_sync_status(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_succeed {
            return Err("Failed to get status".into());
        }

        Ok(SyncStatus {
            last_sync: None,
            total_synced: 42,
            connected_extensions: vec!["mock-extension".to_string()],
            is_running: true,
            connection_status: ExtensionConnectionStatus::Connected as i32,
            error_message: String::new(),
        })
    }

    async fn set_config(&self, _config: &ExtensionConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.should_succeed {
            return Err("Failed to set config".into());
        }

        Ok("Config updated successfully".to_string())
    }
}