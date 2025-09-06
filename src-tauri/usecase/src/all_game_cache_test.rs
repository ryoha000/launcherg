#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use domain::{
        all_game_cache::{
            AllGameCacheOne, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne,
        },
        repository::all_game_cache::MockAllGameCacheRepository,
    };
    use crate::repositorymock::TestRepositories;
    use crate::all_game_cache::AllGameCacheUseCase;

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
                Box::pin(async move {
                    Ok(vec![AllGameCacheOneWithThumbnailUrl {
                    id: 1,
                    gamename: "Test Game 1".to_string(),
                    thumbnail_url: "https://example.com/thumbnail_1.jpg".to_string(),
                    }])
                })
            });

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

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
            .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(vec![]) }));

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

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
                Box::pin(async move {
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
                })
            });

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

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
        mock_repo
            .expect_get_by_ids()
            .with(eq(vec![]))
            .times(1)
            .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Vec::new()) }));

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

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
            Box::pin(async move {
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
            })
        });

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

        let result = use_case.get_all_game_cache().await;
        assert!(result.is_ok());
        let cache = result.unwrap();
        assert_eq!(cache.len(), 2);
    }

    #[tokio::test]
    async fn test_get_cache_last_updated_success() {
        use chrono::Local;
        use std::sync::Arc;

        let mut mock_repo = {
            let mut repo = MockAllGameCacheRepository::new();
            let expected_time = Local::now();
            repo.expect_get_last_updated()
                .times(1)
                .returning(move || Box::pin(async move { Ok::<_, anyhow::Error>((100, expected_time)) }));
            repo
        };

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

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
            .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
        mock_repo.expect_update()
            .with(always())
            .times(1)
            .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
        // update 後に matcher 同期のため get_all が呼ばれる
        mock_repo.expect_get_all()
            .times(1)
            .returning(|| {
                Box::pin(async move {
                    Ok(vec![
                        AllGameCacheOne { id: 1, gamename: "Test Game 1".to_string() },
                        AllGameCacheOne { id: 2, gamename: "Test Game 2".to_string() },
                    ])
                })
            });

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));
        let cache_data = vec![create_test_new_cache_one(1), create_test_new_cache_one(2)];

        let result = use_case.update_all_game_cache(cache_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_all_game_cache_empty() {
        use std::sync::Arc;

        let mut mock_repo = MockAllGameCacheRepository::new();
        // empty cacheの場合、repositoryメソッドは呼ばれない

        let repos = TestRepositories::default();

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));
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
            .returning(|_| Box::pin(async move { Err::<_, anyhow::Error>(anyhow::anyhow!("Database error")) }));

        let mut repos = TestRepositories::default();
        repos.set_all_game_cache(mock_repo);

        let use_case = AllGameCacheUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)));

        let result = use_case.get(1).await;
        assert!(result.is_err());
    }
}
