use super::TestDatabase;
use domain::repository::{RepositoriesExt, works::WorkRepository};
use domain::works::NewWork;

#[tokio::test]
async fn work_repository_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // upsert + find_by_title
    {
        let mut r = repo.work();
        let id = r.upsert(&NewWork { title: "TT".into() }).await.unwrap();
        assert!(id.value > 0);
        let found = r.find_by_title("TT").await.unwrap();
        assert!(found.is_some());
    }
}


