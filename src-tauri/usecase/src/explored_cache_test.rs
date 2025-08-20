#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use std::sync::Arc;

    use crate::{
        domain::{
            explored_cache::ExploredCache, repository::explored_cache::MockExploredCacheRepository,
        },
        usecase::repositorymock::MockRepositoriesExtMock,
        usecase::explored_cache::ExploredCacheUseCase,
    };

    fn create_test_explored_cache() -> ExploredCache {
        vec![
            "/path/to/game1".to_string(),
            "/path/to/game2".to_string(),
            "/path/to/game3".to_string(),
        ]
        .into_iter()
        .collect()
    }

    #[tokio::test]
    async fn test_get_cache_success() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let expected_cache = create_test_explored_cache();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(expected_cache.clone()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_cache().await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert_eq!(cache.len(), 3);
        assert!(cache.contains(&"/path/to/game1".to_string()));
    }

    #[tokio::test]
    async fn test_get_cache_empty() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let empty_cache: ExploredCache = vec![].into_iter().collect();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(empty_cache.clone()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_cache().await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_add_cache_new_paths() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let existing_cache: ExploredCache =
            vec!["/path/to/game1".to_string()].into_iter().collect();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(existing_cache.clone()));
        mock_repo
            .expect_add()
            .with(eq(vec![
                "/path/to/game2".to_string(),
                "/path/to/game3".to_string(),
            ]
            .into_iter()
            .collect::<ExploredCache>()))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));
        let adding_paths = vec![
            "/path/to/game1".to_string(), // already exists, should be filtered out
            "/path/to/game2".to_string(),
            "/path/to/game3".to_string(),
        ];

        let result = use_case.add_cache(adding_paths).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_cache_all_existing() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let existing_cache: ExploredCache =
            vec!["/path/to/game1".to_string(), "/path/to/game2".to_string()]
                .into_iter()
                .collect();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(existing_cache.clone()));
        mock_repo
            .expect_add()
            .with(eq(vec![].into_iter().collect::<ExploredCache>()))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));
        let adding_paths = vec!["/path/to/game1".to_string(), "/path/to/game2".to_string()];

        let result = use_case.add_cache(adding_paths).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_cache_empty_input() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let existing_cache = create_test_explored_cache();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(existing_cache.clone()));
        mock_repo
            .expect_add()
            .with(eq(vec![].into_iter().collect::<ExploredCache>()))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));
        let adding_paths = vec![];

        let result = use_case.add_cache(adding_paths).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_cache_to_empty_cache() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let empty_cache: ExploredCache = vec![].into_iter().collect();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(empty_cache.clone()));
        mock_repo
            .expect_add()
            .with(eq(vec![
                "/path/to/game1".to_string(),
                "/path/to/game2".to_string(),
            ]
            .into_iter()
            .collect::<ExploredCache>()))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));
        let adding_paths = vec!["/path/to/game1".to_string(), "/path/to/game2".to_string()];

        let result = use_case.add_cache(adding_paths).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repository_get_error_propagation() {
        let mut mock_repo = MockExploredCacheRepository::new();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("Database error")));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_cache().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_repository_add_error_propagation() {
        let mut mock_repo = MockExploredCacheRepository::new();
        let existing_cache: ExploredCache = vec![].into_iter().collect();
        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move || Ok(existing_cache.clone()));
        mock_repo
            .expect_add()
            .with(always())
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("Database error")));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_explored_cache_repository()
            .return_const(mock_repo);

        let use_case = ExploredCacheUseCase::new(Arc::new(mock_repositories));
        let adding_paths = vec!["/path/to/game1".to_string()];

        let result = use_case.add_cache(adding_paths).await;
        assert!(result.is_err());
    }
}
