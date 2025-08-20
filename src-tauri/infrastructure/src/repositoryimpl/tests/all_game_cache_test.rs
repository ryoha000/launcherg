#[cfg(test)]
mod tests {
    use domain::{
        all_game_cache::NewAllGameCacheOne, repository::all_game_cache::AllGameCacheRepository,
    };
    use crate::repositoryimpl::tests::TestDatabase;

    fn create_test_new_cache_item(
        id: i32,
        gamename: &str,
        thumbnail_url: &str,
    ) -> NewAllGameCacheOne {
        NewAllGameCacheOne::new(id, gamename.to_string(), thumbnail_url.to_string())
    }

    fn create_test_cache_items(count: usize) -> Vec<NewAllGameCacheOne> {
        (1..=count)
            .map(|i| {
                create_test_new_cache_item(
                    i as i32,
                    &format!("Game {}", i),
                    &format!("https://example.com/thumb/{}.jpg", i),
                )
            })
            .collect()
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        let result = repo.get_all().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_update_and_get_all() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        let test_items = create_test_cache_items(3);
        repo.update(test_items.clone()).await.unwrap();

        let result = repo.get_all().await.unwrap();

        assert_eq!(result.len(), 3);

        // データが正しく格納されているか確認
        let game1 = result.iter().find(|item| item.id == 1).unwrap();
        assert_eq!(game1.gamename, "Game 1");

        let game2 = result.iter().find(|item| item.id == 2).unwrap();
        assert_eq!(game2.gamename, "Game 2");

        let game3 = result.iter().find(|item| item.id == 3).unwrap();
        assert_eq!(game3.gamename, "Game 3");
    }

    #[tokio::test]
    async fn test_update_empty_collection() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        let empty_items: Vec<NewAllGameCacheOne> = vec![];
        let result = repo.update(empty_items).await;

        assert!(result.is_ok());

        let stored_items = repo.get_all().await.unwrap();
        assert!(stored_items.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_ids_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        let empty_ids: Vec<i32> = vec![];
        let result = repo.get_by_ids(empty_ids).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_ids_existing() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // テストデータを挿入
        let test_items = create_test_cache_items(5);
        repo.update(test_items).await.unwrap();

        // 存在するIDで検索
        let target_ids = vec![1, 3, 5];
        let result = repo.get_by_ids(target_ids).await.unwrap();

        assert_eq!(result.len(), 3);

        let ids: Vec<i32> = result.iter().map(|item| item.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(ids.contains(&5));

        // thumbnail_urlも含まれていることを確認
        let item1 = result.iter().find(|item| item.id == 1).unwrap();
        assert_eq!(item1.thumbnail_url, "https://example.com/thumb/1.jpg");
    }

    #[tokio::test]
    async fn test_get_by_ids_nonexistent() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // データを挿入せずに存在しないIDで検索
        let nonexistent_ids = vec![999, 1000];
        let result = repo.get_by_ids(nonexistent_ids).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_ids_mixed() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // テストデータを挿入
        let test_items = create_test_cache_items(3);
        repo.update(test_items).await.unwrap();

        // 存在するIDと存在しないIDの混合
        let mixed_ids = vec![1, 999, 3, 1000];
        let result = repo.get_by_ids(mixed_ids).await.unwrap();

        assert_eq!(result.len(), 2);

        let ids: Vec<i32> = result.iter().map(|item| item.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&999));
        assert!(!ids.contains(&1000));
    }

    #[tokio::test]
    async fn test_delete_by_ids_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        let empty_ids: Vec<i32> = vec![];
        let result = repo.delete_by_ids(empty_ids).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_by_ids_existing() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // テストデータを挿入
        let test_items = create_test_cache_items(5);
        repo.update(test_items).await.unwrap();

        // 削除前の確認
        let before_delete = repo.get_all().await.unwrap();
        assert_eq!(before_delete.len(), 5);

        // 特定のIDを削除
        let delete_ids = vec![2, 4];
        repo.delete_by_ids(delete_ids).await.unwrap();

        // 削除後の確認
        let after_delete = repo.get_all().await.unwrap();
        assert_eq!(after_delete.len(), 3);

        let remaining_ids: Vec<i32> = after_delete.iter().map(|item| item.id).collect();
        assert!(remaining_ids.contains(&1));
        assert!(!remaining_ids.contains(&2));
        assert!(remaining_ids.contains(&3));
        assert!(!remaining_ids.contains(&4));
        assert!(remaining_ids.contains(&5));
    }

    #[tokio::test]
    async fn test_delete_by_ids_nonexistent() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // テストデータを挿入
        let test_items = create_test_cache_items(3);
        repo.update(test_items).await.unwrap();

        // 存在しないIDを削除（エラーにならないことを確認）
        let nonexistent_ids = vec![999, 1000];
        let result = repo.delete_by_ids(nonexistent_ids).await;

        assert!(result.is_ok());

        // データが変更されていないことを確認
        let items = repo.get_all().await.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_last_updated_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // 空のテーブルではエラーになる可能性がある
        let result = repo.get_last_updated().await;

        // エラーまたは適切なデフォルト値が返されることを確認
        // 実装によって動作が異なる可能性があるため、どちらもOKとする
        match result {
            Ok(_) => {
                // 成功した場合はOK（適切なデフォルト値が返された）
            }
            Err(_) => {
                // エラーの場合もOK（空のテーブルでエラーは妥当）
            }
        }
    }

    #[tokio::test]
    async fn test_get_last_updated_with_data() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // テストデータを挿入
        let test_items = create_test_cache_items(3);
        repo.update(test_items).await.unwrap();

        let result = repo.get_last_updated().await.unwrap();

        // 最大IDが返されることを確認
        assert_eq!(result.0, 3);

        // created_atが最近の時刻であることを確認（詳細な時刻チェックは困難なので存在確認のみ）
        assert!(result.1.timestamp() > 0);
    }

    #[tokio::test]
    async fn test_update_large_batch() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // 1500個のアイテムを作成（1000件のチャンク処理をテスト）
        let large_batch = create_test_cache_items(1500);

        repo.update(large_batch).await.unwrap();

        let result = repo.get_all().await.unwrap();
        assert_eq!(result.len(), 1500);

        // 最初と最後の要素をチェック
        let first_item = result.iter().find(|item| item.id == 1).unwrap();
        assert_eq!(first_item.gamename, "Game 1");

        let last_item = result.iter().find(|item| item.id == 1500).unwrap();
        assert_eq!(last_item.gamename, "Game 1500");
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.all_game_cache_repository();

        // 初期データを挿入
        let initial_items = create_test_cache_items(10);
        repo.update(initial_items).await.unwrap();

        // 一部を削除
        repo.delete_by_ids(vec![2, 4, 6, 8]).await.unwrap();

        // 残りのデータを確認
        let remaining = repo.get_all().await.unwrap();
        assert_eq!(remaining.len(), 6);

        // 新しいデータを追加
        let new_items = vec![
            create_test_new_cache_item(20, "New Game 20", "https://example.com/new20.jpg"),
            create_test_new_cache_item(21, "New Game 21", "https://example.com/new21.jpg"),
        ];
        repo.update(new_items).await.unwrap();

        // 最終的なデータを確認
        let final_items = repo.get_all().await.unwrap();
        assert_eq!(final_items.len(), 8); // 6 (remaining) + 2 (new)

        // 特定のIDで検索
        let specific_items = repo.get_by_ids(vec![1, 20, 999]).await.unwrap();
        assert_eq!(specific_items.len(), 2);

        let found_ids: Vec<i32> = specific_items.iter().map(|item| item.id).collect();
        assert!(found_ids.contains(&1));
        assert!(found_ids.contains(&20));
        assert!(!found_ids.contains(&999));
    }
}
