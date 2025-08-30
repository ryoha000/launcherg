use super::TestDatabase;
use domain::repository::{RepositoriesExt, dmm_work_pack::DmmPackRepository, works::WorkRepository};
use domain::works::NewWork;

#[tokio::test]
async fn dmm_work_pack_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // prepare work
    let wid = { let mut r = repo.work(); r.upsert(&NewWork { title: "W".into() }).await.unwrap() };

    // add
    {
        let mut r = repo.dmm_pack();
        r.add(wid.clone()).await.unwrap();
        assert!(r.exists(wid.clone()).await.unwrap());
    }

    // list
    {
        let mut r = repo.dmm_pack();
        let list = r.list().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    // remove
    {
        let mut r = repo.dmm_pack();
        r.remove(wid.clone()).await.unwrap();
        assert!(!r.exists(wid).await.unwrap());
    }
}


