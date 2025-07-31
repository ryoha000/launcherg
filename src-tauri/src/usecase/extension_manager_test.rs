#[cfg(test)]
mod tests {
    use crate::usecase::extension_manager::ExtensionManagerUseCase;
    use crate::{
        infrastructure::repositoryimpl::repository::RepositoriesExt,
        domain::{
            repository::{
                collection::MockCollectionRepository,
                explored_cache::MockExploredCacheRepository,
                all_game_cache::MockAllGameCacheRepository,
            },
            pubsub::PubSubService,
        },
    };
    use std::{path::PathBuf, sync::Arc};
    use tempfile::TempDir;
    use tokio::fs;

    // シンプルなモック実装
    struct MockRepositories {
        collection_repo: MockCollectionRepository,
        explored_cache_repo: MockExploredCacheRepository,
        all_game_cache_repo: MockAllGameCacheRepository,
    }

    impl MockRepositories {
        fn new() -> Self {
            Self {
                collection_repo: MockCollectionRepository::new(),
                explored_cache_repo: MockExploredCacheRepository::new(),
                all_game_cache_repo: MockAllGameCacheRepository::new(),
            }
        }
    }

    impl RepositoriesExt for MockRepositories {
        type CollectionRepo = MockCollectionRepository;
        type ExploredCacheRepo = MockExploredCacheRepository;
        type AllGameCacheRepo = MockAllGameCacheRepository;

        fn collection_repository(&self) -> &MockCollectionRepository { &self.collection_repo }
        fn explored_cache_repository(&self) -> &MockExploredCacheRepository { &self.explored_cache_repo }
        fn all_game_cache_repository(&self) -> &MockAllGameCacheRepository { &self.all_game_cache_repo }
    }

    // モックPubSub実装
    #[derive(Clone)]
    struct MockPubSub;

    impl PubSubService for MockPubSub {
        fn notify<T: serde::Serialize + Clone>(&self, _event: &str, _payload: T) -> Result<(), anyhow::Error> {
            // テスト用なので何もしない
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_extension_manager_with_nonexistent_path() {
        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::with_custom_path(
            repositories,
            pubsub,
            PathBuf::from("/nonexistent/path/native-messaging-host.exe"),
        );

        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "存在しないパスの場合、is_running: falseになるはず");
        assert!(status.connected_extensions.is_empty(), "存在しないパスの場合、接続拡張機能は空のはず");
        assert!(status.last_sync.is_none(), "存在しないパスの場合、last_syncはNoneのはず");
        
        // 詳細状態をチェック
        match status.connection_status {
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostNotFound |
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostStartupFailed => {
                // どちらかのエラー状態になることを期待
            }
            _ => panic!("存在しないパスの場合、HostNotFoundまたはHostStartupFailedになるはず: {:?}", status.connection_status),
        }
        assert!(status.error_message.is_some(), "エラーメッセージが設定されているはず");
    }

    #[tokio::test]
    async fn test_extension_manager_with_invalid_executable() {
        let temp_dir = TempDir::new().unwrap();
        let fake_exe_path = temp_dir.path().join("fake-host.exe");
        
        // 実行可能でないファイルを作成
        fs::write(&fake_exe_path, "not an executable").await.unwrap();

        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::with_custom_path(
            repositories,
            pubsub,
            fake_exe_path,
        );

        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "無効な実行ファイルの場合、is_running: falseになるはず");
        assert!(status.connected_extensions.is_empty());
        
        // 詳細状態をチェック
        assert!(matches!(
            status.connection_status, 
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostStartupFailed
        ), "無効な実行ファイルの場合、HostStartupFailedになるはず: {:?}", status.connection_status);
        assert!(status.error_message.is_some(), "エラーメッセージが設定されているはず");
    }

    #[tokio::test]
    async fn test_extension_manager_with_mock_host() {
        let temp_dir = TempDir::new().unwrap();
        let mock_host_path = temp_dir.path().join("mock-host.exe");
        
        // モックのNative Messaging Hostスクリプトを作成
        let mock_script = create_mock_native_host_script(true);
        fs::write(&mock_host_path, mock_script).await.unwrap();
        
        // Windows の場合、実行権限の設定は不要（.exeなので）
        // Unix系の場合は権限設定が必要だが、このテストはWindows向け

        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::with_custom_path(
            repositories,
            pubsub,
            mock_host_path,
        );

        // このテストは実際のモックスクリプトが動作しないため、失敗することを期待
        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        // 正常なモックスクリプトがないため、is_running: falseになる
        assert!(!status.is_running);
        
        // 詳細状態をチェック（ファイルは存在するが実行に失敗するため）
        assert!(matches!(
            status.connection_status,
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostStartupFailed |
            crate::native_messaging::protocol::ExtensionConnectionStatus::CommunicationError |
            crate::native_messaging::protocol::ExtensionConnectionStatus::HealthCheckTimeout
        ), "モックホストの場合、適切なエラー状態になるはず: {:?}", status.connection_status);
    }

    #[tokio::test]
    async fn test_extension_manager_default_path() {
        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::new(repositories, pubsub);

        let result = extension_manager.check_extension_connection().await;
        
        // デフォルトパスが存在しない場合（開発環境では通常存在しない）
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "デフォルトパスが存在しない場合、is_running: falseになるはず");
        
        // 詳細状態をチェック
        assert!(matches!(
            status.connection_status,
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostNotFound |
            crate::native_messaging::protocol::ExtensionConnectionStatus::HostStartupFailed
        ), "デフォルトパスが存在しない場合、適切なエラー状態になるはず: {:?}", status.connection_status);
    }

    // ensure_process_terminatedはprivateメソッドなので、直接テストできない
    // 代わりにcheck_extension_connectionを通じて間接的にテストされる

    // モックのNative Messaging Hostスクリプトを生成（実際には動作しない）
    fn create_mock_native_host_script(success_response: bool) -> String {
        if success_response {
            r#"
@echo off
echo {"success": true, "data": "OK", "error": null, "request_id": "test"}
"#.to_string()
        } else {
            r#"
@echo off
echo {"success": false, "data": null, "error": "Mock error", "request_id": "test"}
"#.to_string()
        }
    }
}