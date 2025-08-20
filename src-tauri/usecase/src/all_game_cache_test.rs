#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use crate::{
        domain::{
            all_game_cache::{
                AllGameCacheOne, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne,
            },
            repository::all_game_cache::MockAllGameCacheRepository,
        },
        usecase::repositorymock::MockRepositoriesExtMock,
        usecase::all_game_cache::AllGameCacheUseCase,
    };

    fn create_test_new_cache_one(id: i32) -> NewAllGameCacheOne {
        NewAllGameCacheOne {
            id,
            gamename: format!("Test Game {}", id),
            thumbnail_url: format!("https://example.com/thumbnail_{}.jpg", id),
        }
    }

    #[tokio::test]
    async fn test_get_success() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_by_ids()
            .with(eq(vec![1]))
            .times(1)
            .returning(|_| {
                Ok(vec![AllGameCacheOneWithThumbnailUrl {
                    id: 1,
                    gamename: "Test Game 1".to_string(),
                    thumbnail_url: "https://example.com/thumbnail_1.jpg".to_string(),
                }])
            });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get(1).await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert!(cache.is_some());
        assert_eq!(cache.unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_get_not_found() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_by_ids()
            .with(eq(vec![999]))
            .times(1)
            .returning(|_| { Ok(vec![]) });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get(999).await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert!(cache.is_none());
    }

    #[tokio::test]
    async fn test_get_by_ids_success() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_by_ids()
            .with(eq(vec![1, 2]))
            .times(1)
            .returning(|_| {
                Ok(vec![
                    AllGameCacheOneWithThumbnailUrl {
                        id: 1,
                        gamename: "Test Game 1".to_string(),
                        thumbnail_url: "https://example.com/thumbnail_1.jpg".to_string(),
                    },
                    AllGameCacheOneWithThumbnailUrl {
                        id: 2,
                        gamename: "Test Game 2".to_string(),
                        thumbnail_url: "https://example.com/thumbnail_2.jpg".to_string(),
                    },
                ])
            });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_by_ids(vec![1, 2]).await;
        assert!(result.is_ok());
        let caches = result.unwrap();
        assert_eq!(caches.len(), 2);
        assert_eq!(caches[0].id, 1);
        assert_eq!(caches[1].id, 2);
    }

    #[tokio::test]
    async fn test_get_by_ids_empty() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_by_ids()
            .with(eq(vec![]))
            .times(1)
            .returning(|_| { Ok(vec![]) });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_by_ids(vec![]).await;
        assert!(result.is_ok());
        let caches = result.unwrap();
        assert!(caches.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_game_cache_success() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_all().times(1).returning(|| {
            Ok(vec![
                AllGameCacheOne {
                    id: 1,
                    gamename: "Test Game 1".to_string(),
                },
                AllGameCacheOne {
                    id: 2,
                    gamename: "Test Game 2".to_string(),
                },
            ])
        });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_all_game_cache().await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert_eq!(cache.len(), 2);
    }

    #[tokio::test]
    async fn test_get_cache_last_updated_success() {
        use chrono::Local;
        use std::sync::Arc;

        let mock_repo = {
            let mut repo = MockAllGameCacheRepository::new();
            let expected_time = Local::now();
            repo.expect_get_last_updated()
                .times(1)
                .returning(move || { Ok((100, expected_time)) });
            repo
        };

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get_cache_last_updated().await;
        assert!(result.is_ok());
        let (count, _) = result.unwrap();
        assert_eq!(count, 100);
    }

    #[tokio::test]
    async fn test_update_all_game_cache_success() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_delete_by_ids()
            .with(eq(vec![1, 2]))
            .times(1)
            .returning(|_| { Ok(()) });
        mock_repo.expect_update()
            .with(always())
            .times(1)
            .returning(|_| { Ok(()) });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(2) // delete_by_ids and update
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));
        let cache_data = vec![create_test_new_cache_one(1), create_test_new_cache_one(2)];

        let result = use_case.update_all_game_cache(cache_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_all_game_cache_empty() {
        use std::sync::Arc;

        let mock_repo = MockAllGameCacheRepository::new();
        // empty cacheの場合、repositoryメソッドは呼ばれない

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(0) // empty cacheの場合は呼ばれない
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));
        let cache_data = vec![];

        let result = use_case.update_all_game_cache(cache_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repository_error_propagation() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        mock_repo.expect_get_by_ids()
            .with(eq(vec![1]))
            .times(1)
            .returning(|_| { Err(anyhow::anyhow!("Database error")) });

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_all_game_cache_repository()
            .times(1)
            .return_const(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(mock_repositories));

        let result = use_case.get(1).await;
        assert!(result.is_err());
    }
}
