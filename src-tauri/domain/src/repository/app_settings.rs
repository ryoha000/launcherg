use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppStorageSettings {
    pub image_storage_dir: Option<String>,
    pub downloaded_game_storage_dir: Option<String>,
    pub remote_share_device_secret: Option<String>,
    pub remote_share_device_id: Option<String>,
    pub remote_share_server_base_url: Option<String>,
    pub remote_share_last_synced_at: Option<String>,
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait AppSettingsRepository {
    async fn get_storage_settings(&mut self) -> Result<AppStorageSettings>;
    async fn set_storage_settings(&mut self, settings: &AppStorageSettings) -> Result<()>;
}
