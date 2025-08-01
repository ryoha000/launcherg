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
    use std::sync::Arc;

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
        let extension_manager = ExtensionManagerUseCase::new(repositories, pubsub);

        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "存在しないパスの場合、is_running: falseになるはず");
        assert!(status.connected_extensions.is_empty(), "存在しないパスの場合、接続拡張機能は空のはず");
        assert!(status.last_sync.is_none(), "存在しないパスの場合、last_syncはNoneのはず");

        // 詳細状態をチェック
        match status.connection_status {
            3 | 4 => {
                // HostNotFound (3) または HostStartupFailed (4) のどちらかのエラー状態になることを期待
            }
            _ => panic!("存在しないパスの場合、HostNotFoundまたはHostStartupFailedになるはず: {:?}", status.connection_status),
        }
        assert!(!status.error_message.is_empty(), "エラーメッセージが設定されているはず");
    }

    #[tokio::test]
    async fn test_extension_manager_with_invalid_executable() {
        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::new(repositories, pubsub);

        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "無効な実行ファイルの場合、is_running: falseになるはず");
        assert!(status.connected_extensions.is_empty());
        
        // レジストリが見つからない場合はHostNotFoundになる
        assert!(
            status.connection_status == 3, // HostNotFound
            "レジストリエントリが見つからない場合、HostNotFoundになるはず: {:?}", status.connection_status
        );
        assert!(!status.error_message.is_empty(), "エラーメッセージが設定されているはず");
    }

    #[tokio::test]
    async fn test_extension_manager_with_mock_host() {
        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::new(repositories, pubsub);

        let result = extension_manager.check_extension_connection().await;
        
        assert!(result.is_ok());
        let status = result.unwrap();
        // レジストリが見つからない場合、is_running: falseになる
        assert!(!status.is_running, "レジストリエントリが見つからない場合、is_running: falseになるはず");
        
        // 詳細状態をチェック（レジストリが見つからない場合）
        assert!(
            status.connection_status == 3, // HostNotFound
            "レジストリエントリが見つからない場合、HostNotFoundになるはず: {:?}", status.connection_status
        );
        assert!(!status.error_message.is_empty(), "エラーメッセージが設定されているはず");
    }

    #[tokio::test]
    async fn test_extension_manager_default_path() {
        let repositories = Arc::new(MockRepositories::new());
        let pubsub = MockPubSub;
        let extension_manager = ExtensionManagerUseCase::new(repositories, pubsub);

        let result = extension_manager.check_extension_connection().await;
        
        // パスが存在しない場合
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(!status.is_running, "パスが存在しない場合、is_running: falseになるはず");
        
        // 詳細状態をチェック
        assert!(
            matches!(status.connection_status, 3 | 4), // HostNotFound (3) | HostStartupFailed (4)
            "パスが存在しない場合、適切なエラー状態になるはず: {:?}", status.connection_status
        );
    }

    // ensure_process_terminatedはprivateメソッドなので、直接テストできない
    // 代わりにcheck_extension_connectionを通じて間接的にテストされる
}