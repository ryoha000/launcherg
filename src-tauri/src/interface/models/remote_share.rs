#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteShareSettingsVm {
    pub device_secret: Option<String>,
    pub device_id: Option<String>,
    pub server_base_url: Option<String>,
    pub last_remote_sync_at: Option<String>,
}

impl From<domain::repository::app_settings::AppStorageSettings> for RemoteShareSettingsVm {
    fn from(value: domain::repository::app_settings::AppStorageSettings) -> Self {
        Self {
            device_secret: value.remote_share_device_secret,
            device_id: value.remote_share_device_id,
            server_base_url: value.remote_share_server_base_url,
            last_remote_sync_at: value.remote_share_last_synced_at,
        }
    }
}

impl From<RemoteShareSettingsVm> for domain::repository::app_settings::AppStorageSettings {
    fn from(value: RemoteShareSettingsVm) -> Self {
        Self {
            image_storage_dir: None,
            downloaded_game_storage_dir: None,
            remote_share_device_secret: value.device_secret,
            remote_share_device_id: value.device_id,
            remote_share_server_base_url: value.server_base_url,
            remote_share_last_synced_at: value.last_remote_sync_at,
        }
    }
}
