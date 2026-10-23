#[derive(sqlx::FromRow, Clone)]
pub struct AppSettingsTable {
    pub id: i64,
    pub image_storage_dir: Option<String>,
    pub downloaded_game_storage_dir: Option<String>,
    pub created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
}
