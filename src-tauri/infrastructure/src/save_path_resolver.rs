use std::sync::{Arc, RwLock};

use domain::repository::app_settings::AppStorageSettings;
use domain::service::save_path_resolver::SavePathResolver;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoragePathSettings {
    pub image_storage_dir: Option<String>,
    pub downloaded_game_storage_dir: Option<String>,
}

impl From<AppStorageSettings> for StoragePathSettings {
    fn from(value: AppStorageSettings) -> Self {
        Self {
            image_storage_dir: value.image_storage_dir,
            downloaded_game_storage_dir: value.downloaded_game_storage_dir,
        }
    }
}

impl From<StoragePathSettings> for AppStorageSettings {
    fn from(value: StoragePathSettings) -> Self {
        Self {
            image_storage_dir: value.image_storage_dir,
            downloaded_game_storage_dir: value.downloaded_game_storage_dir,
        }
    }
}

#[derive(Debug)]
pub struct StoragePathSettingsStore {
    inner: RwLock<StoragePathSettings>,
}

impl StoragePathSettingsStore {
    pub fn new(initial: StoragePathSettings) -> Self {
        Self {
            inner: RwLock::new(initial),
        }
    }

    pub fn get(&self) -> StoragePathSettings {
        self.inner
            .read()
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn set(&self, settings: StoragePathSettings) {
        if let Ok(mut guard) = self.inner.write() {
            *guard = settings;
        }
    }
}

#[derive(Clone, Debug)]
pub struct DbSavePathResolver {
    root_dir: String,
    settings: Arc<StoragePathSettingsStore>,
}

impl DbSavePathResolver {
    pub fn new(root_dir: String, settings: Arc<StoragePathSettingsStore>) -> Self {
        Self { root_dir, settings }
    }

    fn selected_image_root_dir(&self) -> String {
        self.settings
            .get()
            .image_storage_dir
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| self.root_dir.clone())
    }

    fn selected_downloaded_game_root_dir(&self) -> String {
        self.settings
            .get()
            .downloaded_game_storage_dir
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| self.root_dir.clone())
    }
}

impl SavePathResolver for DbSavePathResolver {
    fn root_dir(&self) -> String {
        self.root_dir.clone()
    }

    fn image_storage_root_dir(&self) -> String {
        self.selected_image_root_dir()
    }

    fn downloaded_game_storage_root_dir(&self) -> String {
        self.selected_downloaded_game_root_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn db_save_path_resolver_未設定時は固定rootを使う() {
        let temp = TempDir::new().unwrap();
        let settings = Arc::new(StoragePathSettingsStore::new(StoragePathSettings::default()));
        let resolver = DbSavePathResolver::new(temp.path().to_string_lossy().to_string(), settings);

        assert!(resolver.icon_png_path("1").contains("game-icons"));
        assert!(resolver.thumbnail_png_path("1").contains("thumbnails"));
        assert!(resolver.downloaded_games_dir().contains("downloaded_games"));
    }

    #[test]
    fn db_save_path_resolver_設定時は可変rootを使う() {
        let temp = TempDir::new().unwrap();
        let image_root = temp.path().join("images");
        let download_root = temp.path().join("downloads");
        let settings = Arc::new(StoragePathSettingsStore::new(StoragePathSettings {
            image_storage_dir: Some(image_root.to_string_lossy().to_string()),
            downloaded_game_storage_dir: Some(download_root.to_string_lossy().to_string()),
        }));
        let resolver = DbSavePathResolver::new(temp.path().to_string_lossy().to_string(), settings);

        assert!(resolver.icon_png_path("1").starts_with(&image_root.to_string_lossy().to_string()));
        assert!(resolver.thumbnail_png_path("1").starts_with(&image_root.to_string_lossy().to_string()));
        assert!(
            resolver.downloaded_games_dir().starts_with(&download_root.to_string_lossy().to_string())
        );
    }
}
