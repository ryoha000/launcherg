#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePathSettingsVm {
    pub image_storage_dir: Option<String>,
    pub downloaded_game_storage_dir: Option<String>,
}

impl From<domain::repository::app_settings::AppStorageSettings> for StoragePathSettingsVm {
    fn from(value: domain::repository::app_settings::AppStorageSettings) -> Self {
        Self {
            image_storage_dir: value.image_storage_dir,
            downloaded_game_storage_dir: value.downloaded_game_storage_dir,
        }
    }
}

impl From<StoragePathSettingsVm> for domain::repository::app_settings::AppStorageSettings {
    fn from(value: StoragePathSettingsVm) -> Self {
        Self {
            image_storage_dir: value.image_storage_dir,
            downloaded_game_storage_dir: value.downloaded_game_storage_dir,
            remote_share_device_secret: None,
            remote_share_device_id: None,
            remote_share_server_base_url: None,
            remote_share_last_synced_at: None,
        }
    }
}
