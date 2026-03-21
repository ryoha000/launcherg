use super::TestDatabase;
use domain::repository::{explored_cache::ExploredCacheRepository, RepositoriesExt};
use std::collections::HashSet;

#[tokio::test]
async fn explored_cache_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // get_all empty
    {
        let mut r = repo.explored_cache();
        let all = r.get_all().await.unwrap();
        assert_eq!(all.len(), 0);
    }

    // add
    {
        let mut r = repo.explored_cache();
        let mut set = HashSet::new();
        set.insert("a".into());
        set.insert("b".into());
        r.add(set).await.unwrap();
    }

    // get_all
    {
        let mut r = repo.explored_cache();
        let all = r.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
