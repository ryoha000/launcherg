use domain::repository::app_settings::{AppSettingsRepository, AppStorageSettings};
use domain::repository::RepositoriesExt;

use super::TestDatabase;

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
            remote_share_device_secret: Some("secret".into()),
            remote_share_device_id: Some("device-id".into()),
            remote_share_server_base_url: Some("https://example.com".into()),
            remote_share_last_synced_at: Some("2026-03-25T00:00:00Z".into()),
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
                remote_share_device_secret: Some("secret".into()),
                remote_share_device_id: Some("device-id".into()),
                remote_share_server_base_url: Some("https://example.com".into()),
                remote_share_last_synced_at: Some("2026-03-25T00:00:00Z".into()),
            }
        );
    }
}
