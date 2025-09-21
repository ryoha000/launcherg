#[cfg(test)]
mod tests {
    use crate::app_signal_router::{endpoint::AppSignalEndpoint, test_support::test_lock};
    use anyhow::Result;

    #[cfg(not(windows))]
    use crate::app_signal_router::test_support::TempDirEnvGuard;
    #[cfg(not(windows))]
    use std::fs;

    #[cfg(not(windows))]
    #[test]
    fn prepare_listener_non_windows_ソケットパスを整備する() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;

        let mut expected_path = env_guard.base_dir().to_path_buf();
        expected_path.push(AppSignalEndpoint::DIR_NAME);
        expected_path.push(AppSignalEndpoint::FILE_NAME);

        if let Some(parent) = expected_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&expected_path, b"stale socket")?;
        assert!(expected_path.exists());

        let config = AppSignalEndpoint::prepare_listener()?;

        assert_eq!(config.cleanup_path.as_ref(), Some(&expected_path));
        assert!(!expected_path.exists());

        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn prepare_listener_windows_ネームスペース付き名前を返す() -> Result<()> {
        let _lock = test_lock();
        let config = AppSignalEndpoint::prepare_listener()?;
        assert!(config.cleanup_path.is_none());

        let expected = AppSignalEndpoint::namespaced_name()?;
        let actual = AppSignalEndpoint::connect_name()?;

        assert_eq!(actual, expected);
        Ok(())
    }

    #[cfg(not(windows))]
    #[test]
    fn connect_name_エンドポイント名を再利用する() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;

        let (expected_name, expected_path) = AppSignalEndpoint::path_name()?;
        let actual_name = AppSignalEndpoint::connect_name()?;
        assert_eq!(actual_name, expected_name);

        let config = AppSignalEndpoint::prepare_listener()?;
        assert_eq!(config.cleanup_path.as_ref(), Some(&expected_path));

        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn connect_name_エンドポイント名を再利用する() -> Result<()> {
        let _lock = test_lock();
        let expected = AppSignalEndpoint::namespaced_name()?;
        let actual = AppSignalEndpoint::connect_name()?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
