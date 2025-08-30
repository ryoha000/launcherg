use super::TestDatabase;
use domain::repository::{RepositoriesExt, work_parent_packs::WorkParentPacksRepository, works::WorkRepository};
use domain::works::NewWork;

#[tokio::test]
async fn work_parent_packs_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // prepare works
    let (wid, pid) = {
        let mut r = repo.work();
        let w = r.upsert(&NewWork { title: "W".into() }).await.unwrap();
        let p = r.upsert(&NewWork { title: "P".into() }).await.unwrap();
        (w, p)
    };

    // add
    {
        let mut r = repo.work_parent_packs();
        r.add(wid.clone(), pid.clone()).await.unwrap();
        assert!(r.exists(wid, pid).await.unwrap());
    }
}


