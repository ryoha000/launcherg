use super::TestDatabase;
use domain::repository::{
    work_parent_packs::WorkParentPacksRepository, works::WorkRepository, RepositoriesExt,
};
use domain::works::NewWork;

#[tokio::test]
async fn work_parent_packs_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    let wid = {
        let mut r = repo.work();
        let w = r.upsert(&NewWork { title: "W".into() }).await.unwrap();
        w
    };
    let parent_key = domain::work_parent_pack::ParentPackKey {
        store_id: "store".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
    };

    // add
    {
        let mut r = repo.work_parent_packs();
        r.add(wid.clone(), parent_key.clone()).await.unwrap();
        assert!(r.exists(wid, parent_key).await.unwrap());
    }
}

#[tokio::test]
async fn work_parent_packs_duplicate_add_is_noop() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    let wid = {
        let mut r = repo.work();
        let w = r.upsert(&NewWork { title: "W".into() }).await.unwrap();
        w
    };
    let parent_key = domain::work_parent_pack::ParentPackKey {
        store_id: "store".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
    };

    {
        let mut r = repo.work_parent_packs();
        r.add(wid.clone(), parent_key.clone()).await.unwrap();
        r.add(wid.clone(), parent_key.clone()).await.unwrap();
        assert!(r.exists(wid.clone(), parent_key.clone()).await.unwrap());
        assert_eq!(r.find_parent_key(wid).await.unwrap().unwrap(), parent_key);
    }
}

#[tokio::test]
async fn work_parent_packs_find_parent_key_should_return_parent() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // prepare work
    let child = {
        let mut r = repo.work();
        let c = r
            .upsert(&NewWork {
                title: "Child".into(),
            })
            .await
            .unwrap();
        c
    };
    let parent_key = domain::work_parent_pack::ParentPackKey {
        store_id: "store".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
    };

    // link and verify
    {
        let mut r = repo.work_parent_packs();
        r.add(child.clone(), parent_key.clone()).await.unwrap();
        let found = r.find_parent_key(child.clone()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap(), parent_key);
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
        let none = r.find_parent_key(orphan).await.unwrap();
        assert!(none.is_none());
    }
}
