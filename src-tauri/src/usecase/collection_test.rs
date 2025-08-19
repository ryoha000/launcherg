#[cfg(test)]
mod tests {
    use chrono::Local;
    use mockall::predicate::*;
    use std::sync::Arc;

    use crate::{
        domain::{
            collection::{
                CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementPaths, CollectionElementThumbnail, NewCollectionElement, NewCollectionElementInfo, ScannedGameElement
            }, repository::collection::MockCollectionRepository, service::save_path_resolver::DirsSavePathResolver, Id
        },
        infrastructure::repositorymock::MockRepositoriesExtMock,
        usecase::{collection::CollectionUseCase, error::UseCaseError},
    };

    fn create_test_element_id(id: i32) -> Id<CollectionElement> {
        Id::new(id)
    }

    fn create_test_new_element(id: i32) -> NewCollectionElement {
        NewCollectionElement::new(create_test_element_id(id), format!("Game {}", id))
    }

    fn create_test_collection_element(id: i32) -> CollectionElement {
        CollectionElement {
            id: create_test_element_id(id),
            gamename: format!("Game {}", id),
            created_at: Local::now(),
            updated_at: Local::now(),
            info: Some(CollectionElementInfo {
                id: Id::new(1),
                collection_element_id: create_test_element_id(id),
                gamename_ruby: "Test Game Ruby".to_string(),
                brandname: "Test Brand".to_string(),
                brandname_ruby: "Test Brand Ruby".to_string(),
                sellday: "2024-01-01".to_string(),
                is_nukige: false,
                created_at: Local::now(),
                updated_at: Local::now(),
            }),
            paths: Some(CollectionElementPaths {
                id: Id::new(1),
                collection_element_id: create_test_element_id(id),
                exe_path: Some("/path/to/game.exe".to_string()),
                lnk_path: Some("/path/to/game.lnk".to_string()),
                created_at: Local::now(),
                updated_at: Local::now(),
            }),
            install: Some(CollectionElementInstall {
                id: Id::new(1),
                collection_element_id: create_test_element_id(id),
                install_at: Local::now(),
                created_at: Local::now(),
                updated_at: Local::now(),
            }),
            play: None,
            like: None,
            thumbnail: Some(CollectionElementThumbnail {
                id: Id::new(1),
                collection_element_id: create_test_element_id(id),
                thumbnail_width: Some(300),
                thumbnail_height: Some(400),
                created_at: Local::now(),
                updated_at: Local::now(),
            }),
            dmm: None,
            dlsite: None,
            erogamescape: None,
        }
    }

    fn create_test_scanned_game_element(id: i32) -> ScannedGameElement {
        ScannedGameElement {
            erogamescape_id: id,
            gamename: format!("Game {}", id),
            exe_path: Some("/path/to/game.exe".to_string()),
            lnk_path: Some("/path/to/game.lnk".to_string()),
            install_at: Some(Local::now()),
        }
    }

    #[tokio::test]
    async fn test_upsert_collection_element_success() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_upsert_collection_element()
            .with(always())
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let element = create_test_new_element(1);

        let result = use_case.upsert_collection_element(&element).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_collection_element_info_success() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_upsert_collection_element_info()
            .with(always())
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let info = NewCollectionElementInfo::new(
            create_test_element_id(1),
            "Test Game Ruby".to_string(),
            "Test Brand".to_string(),
            "Test Brand Ruby".to_string(),
            "2024-01-01".to_string(),
            false,
        );

        let result = use_case.upsert_collection_element_info(&info).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_collection_element_with_full_data() {
        let mut mock_repo = MockCollectionRepository::new();
        // ID解決フェーズ: 既存マッピングなし -> 新規採番 + マッピング登録
        mock_repo
            .expect_get_collection_id_by_erogamescape_id()
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_allocate_new_collection_element_id()
            .times(1)
            .returning(|_| Ok(Id::new(100)));
        mock_repo
            .expect_upsert_erogamescape_map()
            .times(1)
            .returning(|_, _| Ok(()));
        mock_repo
            .expect_upsert_collection_element_paths()
            .with(always())
            .times(1)
            .returning(|_| Ok(()));
        mock_repo
            .expect_upsert_collection_element_install()
            .with(always())
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let element = create_test_scanned_game_element(1);

        let result = use_case.create_collection_element(&element).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_collection_element_minimal_data() {
        let mut mock_repo = MockCollectionRepository::new();
        // ID解決フェーズ: 既存マッピングなし -> 新規採番 + マッピング登録
        mock_repo
            .expect_get_collection_id_by_erogamescape_id()
            .times(1)
            .returning(|_| Ok(None));
        mock_repo
            .expect_allocate_new_collection_element_id()
            .times(1)
            .returning(|_| Ok(Id::new(100)));
        mock_repo
            .expect_upsert_erogamescape_map()
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let element = ScannedGameElement {
            erogamescape_id: 1,
            gamename: "Game 1".to_string(),
            exe_path: None,
            lnk_path: None,
            install_at: None,
        };

        let result = use_case.create_collection_element(&element).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_element_by_element_id_success() {
        let mut mock_repo = MockCollectionRepository::new();
        let expected_element = create_test_collection_element(1);
        mock_repo
            .expect_get_element_by_element_id()
            .with(eq(create_test_element_id(1)))
            .times(1)
            .returning(move |_| Ok(Some(expected_element.clone())));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.get_element_by_element_id(&id).await;
        assert!(result.is_ok());
        let element = result.unwrap();
        assert_eq!(element.id, id);
    }

    #[tokio::test]
    async fn test_get_element_by_element_id_not_found() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_get_element_by_element_id()
            .with(eq(create_test_element_id(1)))
            .times(1)
            .returning(|_| Ok(None));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.get_element_by_element_id(&id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast::<UseCaseError>().unwrap(),
            UseCaseError::CollectionElementIsNotFound
        ));
    }

    #[tokio::test]
    async fn test_update_element_last_play_at_success() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_update_element_last_play_at_by_id()
            .with(eq(create_test_element_id(1)), always())
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.update_element_last_play_at(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_element_like_at_like() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_update_element_like_at_by_id()
            .with(eq(create_test_element_id(1)), always())
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.update_element_like_at(&id, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_element_like_at_unlike() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_update_element_like_at_by_id()
            .with(
                eq(create_test_element_id(1)),
                eq(None::<chrono::DateTime<Local>>),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.update_element_like_at(&id, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_collection_element_by_id_success() {
        let mut mock_repo = MockCollectionRepository::new();
        let expected_element = create_test_collection_element(1);
        mock_repo
            .expect_get_element_by_element_id()
            .with(eq(create_test_element_id(1)))
            .times(1)
            .returning(move |_| Ok(Some(expected_element.clone())));
        mock_repo
            .expect_delete_collection_element()
            .with(eq(create_test_element_id(1)))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.delete_collection_element_by_id(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_collection_element_by_id_not_found() {
        let mut mock_repo = MockCollectionRepository::new();
        mock_repo
            .expect_get_element_by_element_id()
            .with(eq(create_test_element_id(1)))
            .times(1)
            .returning(|_| Ok(None));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));
        let id = create_test_element_id(1);

        let result = use_case.delete_collection_element_by_id(&id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast::<UseCaseError>().unwrap(),
            UseCaseError::CollectionElementIsNotFound
        ));
    }

    #[tokio::test]
    async fn test_get_not_registered_detail_element_ids() {
        let mut mock_repo = MockCollectionRepository::new();
        let expected_ids = vec![create_test_element_id(1), create_test_element_id(2)];
        mock_repo
            .expect_get_not_registered_info_element_ids()
            .times(1)
            .returning(move || Ok(expected_ids.clone()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default()));

        let result = use_case.get_not_registered_detail_element_ids().await;
        assert!(result.is_ok());
        let ids = result.unwrap();
        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_upsert_collection_elements_batch() {
        let mut mock_repo = MockCollectionRepository::new();
        // 各要素ごとにマッピング解決（2回分）
        mock_repo
            .expect_get_collection_id_by_erogamescape_id()
            .times(2)
            .returning(|_| Ok(None));
        mock_repo
            .expect_allocate_new_collection_element_id()
            .times(2)
            .returning(|_| Ok(Id::new(100)));
        mock_repo
            .expect_upsert_erogamescape_map()
            .times(2)
            .returning(|_, _| Ok(()));
        mock_repo
            .expect_upsert_collection_element_paths()
            .times(2)
            .returning(|_| Ok(()));
        mock_repo
            .expect_upsert_collection_element_install()
            .times(2)
            .returning(|_| Ok(()));

        let mut mock_repositories = MockRepositoriesExtMock::new();
        mock_repositories
            .expect_collection_repository()
            .return_const(mock_repo);

        let use_case = CollectionUseCase::new(Arc::new(mock_repositories), Arc::new(DirsSavePathResolver::default())    );
        let elements = vec![
            create_test_scanned_game_element(1),
            create_test_scanned_game_element(2),
        ];

        let result = use_case.upsert_collection_elements(&elements).await;
        assert!(result.is_ok());
    }
}
