use super::TestDatabase;
use domain::repository::{RepositoriesExt, work_omit::WorkOmitRepository, works::WorkRepository};
use domain::works::NewWork;

#[tokio::test]
async fn work_omit_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // create work
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork { title: "W".into() }).await.unwrap()
    };

    // add
    {
        let mut r = repo.work_omit();
        r.add(work_id.clone()).await.unwrap();
        assert!(r.exists(work_id.clone()).await.unwrap());
    }

    // list
    {
        let mut r = repo.work_omit();
        let list = r.list().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    // remove
    {
        let mut r = repo.work_omit();
        r.remove(work_id.clone()).await.unwrap();
        assert!(!r.exists(work_id).await.unwrap());
    }
}


