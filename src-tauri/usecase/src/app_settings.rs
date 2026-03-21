use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use domain::repository::app_settings::{AppSettingsRepository, AppStorageSettings};
use domain::repository::manager::RepositoryManager;
use domain::repository::RepositoriesExt;

pub struct AppSettingsUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: PhantomData<R>,
}

impl<M, R> AppSettingsUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>) -> Self {
        Self {
            manager,
            _marker: PhantomData,
        }
    }

    pub async fn get_storage_settings(&self) -> anyhow::Result<AppStorageSettings> {
        self.manager
            .run(|repos| Box::pin(async move { repos.app_settings().get_storage_settings().await }))
            .await
    }

    pub async fn set_storage_settings(
        &self,
        settings: AppStorageSettings,
    ) -> anyhow::Result<AppStorageSettings> {
        let normalized = AppStorageSettings {
            image_storage_dir: normalize_optional_path(settings.image_storage_dir),
            downloaded_game_storage_dir: normalize_optional_path(settings.downloaded_game_storage_dir),
        };
        validate_storage_dir(normalized.image_storage_dir.as_deref())?;
        validate_storage_dir(normalized.downloaded_game_storage_dir.as_deref())?;

        self.manager
            .run(|repos| {
                let normalized = normalized.clone();
                Box::pin(async move {
                    repos.app_settings().set_storage_settings(&normalized).await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;

        Ok(normalized)
    }
}

fn normalize_optional_path(path: Option<String>) -> Option<String> {
    path.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn validate_storage_dir(path: Option<&str>) -> anyhow::Result<()> {
    let Some(path) = path else {
        return Ok(());
    };

    let candidate = Path::new(path);
    if !candidate.is_absolute() {
        return Err(anyhow::anyhow!("storage dir must be an absolute path: {}", path));
    }

    std::fs::create_dir_all(candidate)?;
    let probe = candidate.join(format!(
        ".launcherg-write-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    {
        let _file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&probe)?;
    }
    std::fs::remove_file(&probe).ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::app_settings::AppStorageSettings;
    use domain::repository::mock::{TestRepositories, TestRepositoryManager};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn set_storage_settings_空文字は_null_に正規化される() {
        let repos = TestRepositories::default();
        {
            let mut app_settings = repos.app_settings.lock().await;
            app_settings
                .expect_set_storage_settings()
                .returning(|settings| {
                    assert_eq!(settings.image_storage_dir, None);
                    assert_eq!(settings.downloaded_game_storage_dir, None);
                    Box::pin(async { Ok::<_, anyhow::Error>(()) })
                });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let usecase = AppSettingsUseCase::new(manager);

        let saved = usecase
            .set_storage_settings(AppStorageSettings {
                image_storage_dir: Some("".into()),
                downloaded_game_storage_dir: Some("   ".into()),
            })
            .await
            .unwrap();

        assert_eq!(saved, AppStorageSettings::default());
    }

    #[tokio::test]
    async fn set_storage_settings_相対パスは失敗する() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let usecase = AppSettingsUseCase::new(manager);

        let err = usecase
            .set_storage_settings(AppStorageSettings {
                image_storage_dir: Some("relative/path".into()),
                downloaded_game_storage_dir: None,
            })
            .await
            .unwrap_err();

        assert!(err.to_string().contains("absolute path"));
    }

    #[tokio::test]
    async fn set_storage_settings_絶対パスは保存される() {
        let temp = TempDir::new().unwrap();
        let repos = TestRepositories::default();
        {
            let mut app_settings = repos.app_settings.lock().await;
            app_settings
                .expect_set_storage_settings()
                .returning(|settings| {
                    assert!(settings.image_storage_dir.is_some());
                    assert!(settings.downloaded_game_storage_dir.is_some());
                    Box::pin(async { Ok::<_, anyhow::Error>(()) })
                });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let usecase = AppSettingsUseCase::new(manager);

        let saved = usecase
            .set_storage_settings(AppStorageSettings {
                image_storage_dir: Some(temp.path().join("images").to_string_lossy().to_string()),
                downloaded_game_storage_dir: Some(
                    temp.path()
                        .join("downloads")
                        .to_string_lossy()
                        .to_string(),
                ),
            })
            .await
            .unwrap();

        assert!(saved.image_storage_dir.is_some());
        assert!(saved.downloaded_game_storage_dir.is_some());
    }
}
