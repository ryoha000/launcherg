use super::TestDatabase;
use domain::repository::{
    work_parent_packs::WorkParentPacksRepository, works::WorkRepository, RepositoriesExt,
};
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

#[tokio::test]
async fn work_parent_packs_find_parent_id_should_return_parent() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // prepare works
    let (child, parent) = {
        let mut r = repo.work();
        let c = r
            .upsert(&NewWork {
                title: "Child".into(),
            })
            .await
            .unwrap();
        let p = r
            .upsert(&NewWork {
                title: "Parent".into(),
            })
            .await
            .unwrap();
        (c, p)
    };

    // link and verify
    {
        let mut r = repo.work_parent_packs();
        r.add(child.clone(), parent.clone()).await.unwrap();
        let found = r.find_parent_id(child.clone()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().value, parent.value);
    }

    // no link case
    {
        let mut r = repo.work_parent_packs();
        let orphan = repo
            .work()
            .upsert(&NewWork {
                title: "Orphan".into(),
            })
            .await
            .unwrap();
        let none = r.find_parent_id(orphan).await.unwrap();
        assert!(none.is_none());
    }
}
