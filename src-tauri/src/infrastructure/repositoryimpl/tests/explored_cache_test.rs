#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        domain::repository::explored_cache::ExploredCacheRepository,
        infrastructure::repositoryimpl::tests::TestDatabase,
    };

    #[tokio::test]
    async fn test_get_all_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        let result = repo.get_all().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_get_all() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        let test_paths: HashSet<String> = [
            "/test/path/1".to_string(),
            "/test/path/2".to_string(),
            "/test/path/3".to_string(),
        ]
        .into_iter()
        .collect();

        repo.add(test_paths.clone()).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&"/test/path/1".to_string()));
        assert!(result.contains(&"/test/path/2".to_string()));
        assert!(result.contains(&"/test/path/3".to_string()));
    }

    #[tokio::test]
    async fn test_add_empty_collection() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        let empty_paths: HashSet<String> = HashSet::new();
        let result = repo.add(empty_paths).await;

        assert!(result.is_ok());

        let stored_paths = repo.get_all().await.unwrap();
        assert!(stored_paths.is_empty());
    }

    #[tokio::test]
    async fn test_add_multiple_batches() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        // 最初のバッチを追加
        let first_batch: HashSet<String> =
            ["/batch1/path1".to_string(), "/batch1/path2".to_string()]
                .into_iter()
                .collect();
        repo.add(first_batch).await.unwrap();

        // 2番目のバッチを追加
        let second_batch: HashSet<String> = [
            "/batch2/path1".to_string(),
            "/batch2/path2".to_string(),
            "/batch2/path3".to_string(),
        ]
        .into_iter()
        .collect();
        repo.add(second_batch).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 5);
        assert!(result.contains(&"/batch1/path1".to_string()));
        assert!(result.contains(&"/batch1/path2".to_string()));
        assert!(result.contains(&"/batch2/path1".to_string()));
        assert!(result.contains(&"/batch2/path2".to_string()));
        assert!(result.contains(&"/batch2/path3".to_string()));
    }

    #[tokio::test]
    async fn test_add_single_path() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        let single_path: HashSet<String> = ["/single/test/path".to_string()].into_iter().collect();
        repo.add(single_path).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains("/single/test/path"));
    }

    #[tokio::test]
    async fn test_add_paths_with_special_characters() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        let special_paths: HashSet<String> = [
            "/path with spaces/file.txt".to_string(),
            "/path/with/unicode/テスト.txt".to_string(),
            "C:\\Windows\\Path\\file.exe".to_string(),
        ]
        .into_iter()
        .collect();

        repo.add(special_paths.clone()).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 3);
        for path in special_paths {
            assert!(result.contains(&path));
        }
    }

    #[tokio::test]
    async fn test_add_large_batch() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.explored_cache_repository();

        // 1000個のパスを生成
        let large_batch: HashSet<String> = (0..1000)
            .map(|i| format!("/large/batch/path/{}", i))
            .collect();

        repo.add(large_batch.clone()).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 1000);

        // いくつかのランダムなパスをチェック
        assert!(result.contains(&"/large/batch/path/0".to_string()));
        assert!(result.contains(&"/large/batch/path/500".to_string()));
        assert!(result.contains(&"/large/batch/path/999".to_string()));
    }
}
