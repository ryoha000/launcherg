#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            collection::{
                NewCollectionElement, NewCollectionElementInfo, NewCollectionElementPaths,
            },
            repository::collection::CollectionRepository,
            Id,
        },
        infrastructure::repositoryimpl::tests::TestDatabase,
    };

    fn create_test_collection_element_id(
        id: i32,
    ) -> Id<crate::domain::collection::CollectionElement> {
        Id::new(id)
    }

    fn create_test_new_collection_element(id: i32) -> NewCollectionElement {
        NewCollectionElement::new(create_test_collection_element_id(id), format!("Game {}", id))
    }

    fn create_test_new_collection_element_info(
        element_id: i32,
        gamename: &str,
    ) -> NewCollectionElementInfo {
        NewCollectionElementInfo::new(
            create_test_collection_element_id(element_id),
            format!("{}_ruby", gamename),
            "Test Brand".to_string(),
            "Test Brand Ruby".to_string(),
            "2024-01-01".to_string(),
            false,
        )
    }

    fn create_test_new_collection_element_paths(
        element_id: i32,
        exe_path: Option<String>,
        lnk_path: Option<String>,
    ) -> NewCollectionElementPaths {
        NewCollectionElementPaths::new(
            create_test_collection_element_id(element_id),
            exe_path,
            lnk_path,
        )
    }

    #[tokio::test]
    async fn test_get_all_elements_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        let result = repo.get_all_elements().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_upsert_and_get_collection_element() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        let new_element = create_test_new_collection_element(1);

        // 要素を挿入
        repo.upsert_collection_element(&new_element).await.unwrap();

        // 全要素を取得して確認
        let elements = repo.get_all_elements().await.unwrap();
        assert_eq!(elements.len(), 1);

        let element = &elements[0];
        assert_eq!(element.id.value, 1);

        // 関連データは空であることを確認
        assert!(element.info.is_none());
        assert!(element.paths.is_none());
        assert!(element.install.is_none());
        assert!(element.play.is_none());
        assert!(element.like.is_none());
        assert!(element.thumbnail.is_none());
    }

    #[tokio::test]
    async fn test_get_element_by_id_existing() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        let new_element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&new_element).await.unwrap();

        let element_id = create_test_collection_element_id(1);
        let result = repo.get_element_by_element_id(&element_id).await.unwrap();

        assert!(result.is_some());
        let element = result.unwrap();
        assert_eq!(element.id.value, 1);
    }

    #[tokio::test]
    async fn test_get_element_by_id_nonexistent() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        let element_id = create_test_collection_element_id(999);
        let result = repo.get_element_by_element_id(&element_id).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_collection_element() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        let new_element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&new_element).await.unwrap();

        // 削除前の確認
        let elements_before = repo.get_all_elements().await.unwrap();
        assert_eq!(elements_before.len(), 1);

        // 削除
        let element_id = create_test_collection_element_id(1);
        repo.delete_collection_element(&element_id).await.unwrap();

        // 削除後の確認
        let elements_after = repo.get_all_elements().await.unwrap();
        assert!(elements_after.is_empty());
    }

    #[tokio::test]
    async fn test_upsert_collection_element_info() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // まず要素を作成
        let new_element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&new_element).await.unwrap();

        // 情報を追加
        let new_info = create_test_new_collection_element_info(1, "Test Game");
        repo.upsert_collection_element_info(&new_info)
            .await
            .unwrap();

        // 情報が正しく取得できることを確認
        let element_id = create_test_collection_element_id(1);
        let info = repo
            .get_element_info_by_element_id(&element_id)
            .await
            .unwrap();

        assert!(info.is_some());
        let info = info.unwrap();
        // gamename は CollectionElement 側に移行済み
        assert_eq!(info.gamename_ruby, "Test Game_ruby");
        assert_eq!(info.brandname, "Test Brand");
        assert!(!info.is_nukige);
    }

    #[tokio::test]
    async fn test_upsert_collection_element_info_update() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // まず要素を作成
        let new_element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&new_element).await.unwrap();

        // 初期情報を追加
        let new_info = create_test_new_collection_element_info(1, "Original Game");
        repo.upsert_collection_element_info(&new_info)
            .await
            .unwrap();

        // 情報を更新
        let updated_info = create_test_new_collection_element_info(1, "Updated Game");
        repo.upsert_collection_element_info(&updated_info)
            .await
            .unwrap();

        // 更新された情報が取得できることを確認
        let element_id = create_test_collection_element_id(1);
        let info = repo
            .get_element_info_by_element_id(&element_id)
            .await
            .unwrap();

        assert!(info.is_some());
        let info = info.unwrap();
        // gamename は CollectionElement 側に移行済み
        assert_eq!(info.gamename_ruby, "Updated Game_ruby");
    }

    #[tokio::test]
    async fn test_upsert_collection_element_paths() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // まず要素を作成
        let new_element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&new_element).await.unwrap();

        // パス情報を追加
        let new_paths = create_test_new_collection_element_paths(
            1,
            Some("C:\\test\\game.exe".to_string()),
            Some("C:\\test\\game.lnk".to_string()),
        );
        repo.upsert_collection_element_paths(&new_paths)
            .await
            .unwrap();

        // パス情報が正しく取得できることを確認
        let element_id = create_test_collection_element_id(1);
        let paths = repo
            .get_element_paths_by_element_id(&element_id)
            .await
            .unwrap();

        assert!(paths.is_some());
        let paths = paths.unwrap();
        assert_eq!(paths.exe_path, Some("C:\\test\\game.exe".to_string()));
        assert_eq!(paths.lnk_path, Some("C:\\test\\game.lnk".to_string()));
    }

    #[tokio::test]
    async fn test_get_not_registered_info_element_ids() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // 情報がない要素を作成
        let element1 = create_test_new_collection_element(1);
        let element2 = create_test_new_collection_element(2);
        repo.upsert_collection_element(&element1).await.unwrap();
        repo.upsert_collection_element(&element2).await.unwrap();

        // 1つ目にだけ情報を追加
        let info1 = create_test_new_collection_element_info(1, "Game 1");
        repo.upsert_collection_element_info(&info1).await.unwrap();

        // 情報が登録されていない要素のIDを取得
        let unregistered_ids = repo.get_not_registered_info_element_ids().await.unwrap();

        assert_eq!(unregistered_ids.len(), 1);
        assert_eq!(unregistered_ids[0].value, 2);
    }

    #[tokio::test]
    async fn test_multiple_elements_with_relations() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // 複数の要素を作成
        for i in 1..=3 {
            let element = create_test_new_collection_element(i);
            repo.upsert_collection_element(&element).await.unwrap();

            let info = create_test_new_collection_element_info(i, &format!("Game {}", i));
            repo.upsert_collection_element_info(&info).await.unwrap();

            let paths = create_test_new_collection_element_paths(
                i,
                Some(format!("C:\\game{}\\game.exe", i)),
                None,
            );
            repo.upsert_collection_element_paths(&paths).await.unwrap();
        }

        // 全要素を取得して関連データが正しく設定されていることを確認
        let elements = repo.get_all_elements().await.unwrap();
        assert_eq!(elements.len(), 3);

        for (i, element) in elements.iter().enumerate() {
            let expected_id = (i + 1) as i32;
            assert_eq!(element.id.value, expected_id);

            // 情報が設定されていることを確認（gamename は CollectionElement 側）
            assert!(element.info.is_some());
            let info = element.info.as_ref().unwrap();
            assert_eq!(info.gamename_ruby, format!("Game {}_ruby", expected_id));
            assert_eq!(info.brandname, "Test Brand");
            assert_eq!(info.brandname_ruby, "Test Brand Ruby");
            assert_eq!(info.sellday, "2024-01-01");
            assert!(!info.is_nukige);

            // パス情報が設定されていることを確認
            assert!(element.paths.is_some());
            let paths = element.paths.as_ref().unwrap();
            assert_eq!(
                paths.exe_path,
                Some(format!("C:\\game{}\\game.exe", expected_id))
            );

            // その他の関連データは空であることを確認
            assert!(element.install.is_none());
            assert!(element.play.is_none());
            assert!(element.like.is_none());
            assert!(element.thumbnail.is_none());
        }
    }

    #[tokio::test]
    async fn test_element_with_complete_join_query() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.collection_repository();

        // 要素を作成
        let element = create_test_new_collection_element(1);
        repo.upsert_collection_element(&element).await.unwrap();

        // 情報を追加
        let info = create_test_new_collection_element_info(1, "Complete Game");
        repo.upsert_collection_element_info(&info).await.unwrap();

        // パス情報を追加
        let paths = create_test_new_collection_element_paths(
            1,
            Some("C:\\complete\\game.exe".to_string()),
            Some("C:\\complete\\game.lnk".to_string()),
        );
        repo.upsert_collection_element_paths(&paths).await.unwrap();

        // get_element_by_element_idで複雑なJOINクエリをテスト
        let element_id = create_test_collection_element_id(1);
        let result = repo.get_element_by_element_id(&element_id).await.unwrap();

        assert!(result.is_some());
        let element = result.unwrap();

        // メイン要素の確認
        assert_eq!(element.id.value, 1);

        // JOINされた情報の確認（gamename は CollectionElement 側）
        assert!(element.info.is_some());
        let info = element.info.as_ref().unwrap();
        assert_eq!(info.gamename_ruby, "Complete Game_ruby");
        assert_eq!(info.brandname, "Test Brand");
        assert_eq!(info.brandname_ruby, "Test Brand Ruby");
        assert_eq!(info.sellday, "2024-01-01");
        assert!(!info.is_nukige);

        assert!(element.paths.is_some());
        let paths = element.paths.as_ref().unwrap();
        assert_eq!(paths.exe_path, Some("C:\\complete\\game.exe".to_string()));
        assert_eq!(paths.lnk_path, Some("C:\\complete\\game.lnk".to_string()));
    }
}
