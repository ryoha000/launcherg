use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcTailVersion {
    pub version: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcTailManagerStatus {
    pub current_version: Option<String>,
    pub is_running: bool,
    pub executable_exists: bool,
    pub update_available: bool,
}

#[derive(Debug, Error)]
pub enum ProcTailManagerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ProcTail process error: {0}")]
    Process(String),
    #[error("Download error: {0}")]
    Download(String),
}

// ProcTailManager のドメイン側トレイト定義（trait-variantで生成）
#[trait_variant::make(Send)]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait ProcTailManagerTrait {
    async fn get_status(&self) -> Result<ProcTailManagerStatus, ProcTailManagerError>;
    async fn get_latest_version(&self) -> Result<ProcTailVersion, ProcTailManagerError>;
    async fn is_update_available(&self) -> Result<bool, ProcTailManagerError>;
    async fn download_and_install(
        &self,
        version: &ProcTailVersion,
    ) -> Result<(), ProcTailManagerError>;
    async fn start_proctail(&self) -> Result<(), ProcTailManagerError>;
    async fn stop_proctail(&self) -> Result<(), ProcTailManagerError>;
    async fn is_running(&self) -> bool;
}

