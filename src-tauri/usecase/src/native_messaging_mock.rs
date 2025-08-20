#[cfg(any(test, feature = "mocks"))]
mockall::mock! {
    pub NativeMessagingHostClient {}

    impl crate::domain::extension::NativeMessagingHostClient for NativeMessagingHostClient {
        async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
        async fn get_sync_status(&self) -> Result<crate::domain::extension::SyncStatus, Box<dyn std::error::Error + Send + Sync>>;
        async fn set_config(&self, config: &crate::domain::extension::ExtensionConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    }
}


