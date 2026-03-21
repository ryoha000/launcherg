use crate::sqliterepository::{
    models::app_settings::AppSettingsTable, sqliterepository::RepositoryImpl,
};
use domain::repository::app_settings::{AppSettingsRepository, AppStorageSettings};

impl AppSettingsRepository for RepositoryImpl<AppStorageSettings> {
    async fn get_storage_settings(&mut self) -> anyhow::Result<AppStorageSettings> {
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<AppSettingsTable> = sqlx::query_as(
                        r#"SELECT id, image_storage_dir, downloaded_game_storage_dir, created_at, updated_at FROM app_settings WHERE id = 1 LIMIT 1"#,
                    )
                    .fetch_optional(conn)
                    .await?;
                    let row = row.unwrap_or(AppSettingsTable {
                        id: 1,
                        image_storage_dir: None,
                        downloaded_game_storage_dir: None,
                        created_at: None,
                        updated_at: None,
                    });
                    Ok::<AppStorageSettings, anyhow::Error>(AppStorageSettings {
                        image_storage_dir: row.image_storage_dir,
                        downloaded_game_storage_dir: row.downloaded_game_storage_dir,
                    })
                })
            })
            .await
    }

    async fn set_storage_settings(&mut self, settings: &AppStorageSettings) -> anyhow::Result<()> {
        let image_storage_dir = settings.image_storage_dir.clone();
        let downloaded_game_storage_dir = settings.downloaded_game_storage_dir.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
                        INSERT INTO app_settings (id, image_storage_dir, downloaded_game_storage_dir)
                        VALUES (1, ?, ?)
                        ON CONFLICT(id) DO UPDATE SET
                            image_storage_dir = excluded.image_storage_dir,
                            downloaded_game_storage_dir = excluded.downloaded_game_storage_dir,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(image_storage_dir)
                    .bind(downloaded_game_storage_dir)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
    }
}
