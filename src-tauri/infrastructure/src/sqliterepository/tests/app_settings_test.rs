use domain::repository::app_settings::{AppSettingsRepository, AppStorageSettings};
use domain::repository::RepositoriesExt;

use super::TestDatabase;

#[tokio::test]
async fn app_settings_共有設定の削除後も保存先と作品を維持する() {
    let test_db = TestDatabase::new_with_target(refinery::Target::Version(5))
        .await
        .unwrap();
    sqlx::query(
        "UPDATE app_settings SET image_storage_dir = ?, downloaded_game_storage_dir = ?,
         remote_share_device_secret = ?, remote_share_device_id = ?,
         remote_share_server_base_url = ?, remote_share_last_synced_at = ? WHERE id = 1",
    )
    .bind("C:/images")
    .bind("D:/downloads")
    .bind("retired-secret")
    .bind("retired-device")
    .bind("https://example.com")
    .bind("2026-03-25T00:00:00Z")
    .execute(&test_db.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO works (id, title) VALUES (?, ?)")
        .bind("existing-work")
        .bind("既存の作品")
        .execute(&test_db.pool)
        .await
        .unwrap();

    let mut config = refinery::config::Config::new(refinery::config::ConfigDbType::Sqlite)
        .set_db_path(test_db._temp_file.path().to_str().unwrap());
    super::embedded::migrations::runner()
        .run(&mut config)
        .unwrap();

    let repo = test_db.sqlite_repository();
    let settings = repo.app_settings().get_storage_settings().await.unwrap();
    assert_eq!(
        settings,
        AppStorageSettings {
            image_storage_dir: Some("C:/images".into()),
            downloaded_game_storage_dir: Some("D:/downloads".into()),
        }
    );
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('app_settings')")
            .fetch_all(&test_db.pool)
            .await
            .unwrap();
    assert_eq!(
        columns,
        [
            "id",
            "image_storage_dir",
            "downloaded_game_storage_dir",
            "created_at",
            "updated_at",
        ]
    );
    let title: String = sqlx::query_scalar("SELECT title FROM works WHERE id = 'existing-work'")
        .fetch_one(&test_db.pool)
        .await
        .unwrap();
    assert_eq!(title, "既存の作品");
}

#[tokio::test]
async fn app_settings_repository_get_set_一連の操作() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    {
        let mut r = repo.app_settings();
        let initial = r.get_storage_settings().await.unwrap();
        assert_eq!(initial, AppStorageSettings::default());
    }

    {
        let mut r = repo.app_settings();
        r.set_storage_settings(&AppStorageSettings {
            image_storage_dir: Some("C:/images".into()),
            downloaded_game_storage_dir: Some("D:/downloads".into()),
        })
        .await
        .unwrap();
    }

    {
        let mut r = repo.app_settings();
        let current = r.get_storage_settings().await.unwrap();
        assert_eq!(
            current,
            AppStorageSettings {
                image_storage_dir: Some("C:/images".into()),
                downloaded_game_storage_dir: Some("D:/downloads".into()),
            }
        );
    }
}
