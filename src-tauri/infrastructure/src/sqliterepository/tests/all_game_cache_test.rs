use super::TestDatabase;
use domain::all_game_cache::NewAllGameCacheOne;
use domain::repository::{all_game_cache::AllGameCacheRepository, RepositoriesExt};

#[tokio::test]
async fn all_game_cache_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // insert batch
    {
        let mut r = repo.all_game_cache();
        r.update(vec![
            NewAllGameCacheOne::new(1, "game1".into(), "http://example.com/1.png".into()),
            NewAllGameCacheOne::new(2, "game2".into(), "http://example.com/2.png".into()),
        ])
        .await
        .unwrap();
    }

    // get_all
    {
        let mut r = repo.all_game_cache();
        let all = r.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    // get_by_ids
    {
        let mut r = repo.all_game_cache();
        let rows = r.get_by_ids(vec![1, 2]).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows
            .iter()
            .any(|v| v.id == 1 && v.thumbnail_url.ends_with("1.png")));
        assert!(rows
            .iter()
            .any(|v| v.id == 2 && v.thumbnail_url.ends_with("2.png")));
    }

    // get_last_updated
    {
        let mut r = repo.all_game_cache();
        let (id, _ts) = r.get_last_updated().await.unwrap();
        assert!(id >= 2);
    }

    // search_by_name
    {
        let mut r = repo.all_game_cache();
        let rows = r.search_by_name("game").await.unwrap();
        assert_eq!(rows.len(), 2);
    }

    // delete_by_ids
    {
        let mut r = repo.all_game_cache();
        r.delete_by_ids(vec![1]).await.unwrap();
        let rest = r.get_all().await.unwrap();
        assert_eq!(rest.len(), 1);
    }
}
