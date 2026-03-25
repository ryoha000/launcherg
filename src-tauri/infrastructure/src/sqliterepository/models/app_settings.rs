#[derive(sqlx::FromRow, Clone)]
pub struct AppSettingsTable {
    pub id: i64,
    pub image_storage_dir: Option<String>,
    pub downloaded_game_storage_dir: Option<String>,
    pub remote_share_device_secret: Option<String>,
    pub remote_share_device_id: Option<String>,
    pub remote_share_server_base_url: Option<String>,
    pub remote_share_last_synced_at: Option<String>,
    pub created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
}
