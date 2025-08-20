#[cfg(test)]
mockall::mock! {
    pub NativeMessagingHostClient {}

    impl domain::extension::NativeMessagingHostClient for NativeMessagingHostClient {
        async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
        async fn get_sync_status(&self) -> Result<domain::extension::SyncStatus, Box<dyn std::error::Error + Send + Sync>>;
        async fn set_config(&self, config: &domain::extension::ExtensionConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    }
}


