use super::TestDatabase;
use domain::repositoryv2::{RepositoriesExt, works::{DmmWorkRepository, DlsiteWorkRepository, WorkRepository}};
use domain::works::{NewDmmWork, NewDlsiteWork};

#[tokio::test]
async fn dmm_works_upsert_and_find_by_store_key() {
    let test_db = TestDatabase::new().await.unwrap();
    let mut repo = test_db.sqlite_repository();
    let repo = repo.dmm_work();

    let id = DmmWorkRepository::upsert(repo, &NewDmmWork {
        title: "Title A".into(),
        store_id: "SID-1".into(),
        category: "software".into(),
        subcategory: "game".into(),
    }).await.unwrap();
    assert!(id.value > 0);

    let found = DmmWorkRepository::find_by_store_key(repo, "SID-1", "software", "game").await.unwrap();
    assert!(found.is_some());
    let w = found.unwrap();
    assert_eq!(w.store_id, "SID-1");
    assert_eq!(w.category, "software");
    assert_eq!(w.subcategory, "game");

    let id2 = DmmWorkRepository::upsert(repo, &NewDmmWork {
        title: "Title A2".into(),
        store_id: "SID-1".into(),
        category: "software".into(),
        subcategory: "utility".into(),
    }).await.unwrap();
    assert_eq!(id.value, id2.value);

    let updated = DmmWorkRepository::find_by_store_key(repo, "SID-1", "software", "utility").await.unwrap();
    assert!(updated.is_some());
    let w2 = updated.unwrap();
    assert_eq!(w2.title, "Title A2");
    assert_eq!(w2.subcategory, "utility");
}

#[tokio::test]
async fn dlsite_works_upsert_and_find_by_store_key() {
    let test_db = TestDatabase::new().await.unwrap();
    let mut repo = test_db.sqlite_repository();
    let repo = repo.dlsite_work();

    let id = DlsiteWorkRepository::upsert(repo, &NewDlsiteWork {
        title: "DL Title".into(),
        store_id: "RJ123".into(),
        category: "software".into(),
    }).await.unwrap();
    assert!(id.value > 0);

    let found = DlsiteWorkRepository::find_by_store_key(repo, "RJ123", "software").await.unwrap();
    assert!(found.is_some());
    let w = found.unwrap();
    assert_eq!(w.store_id, "RJ123");
    assert_eq!(w.category, "software");

    let id2 = DlsiteWorkRepository::upsert(repo, &NewDlsiteWork {
        title: "DL Title 2".into(),
        store_id: "RJ123".into(),
        category: "doujin".into(),
    }).await.unwrap();
    assert_eq!(id.value, id2.value);

    let updated = DlsiteWorkRepository::find_by_store_key(repo, "RJ123", "doujin").await.unwrap();
    assert!(updated.is_some());
    let w2 = updated.unwrap();
    assert_eq!(w2.title, "DL Title 2");
    assert_eq!(w2.category, "doujin");
}

#[tokio::test]
async fn list_all_details_dmm_only() {
    let test_db = TestDatabase::new().await.unwrap();
    let mut repo = test_db.sqlite_repository();
    {
        let dmm_repo = repo.dmm_work();
        let _ = DmmWorkRepository::upsert(dmm_repo, &NewDmmWork {
            title: "Title A".into(),
            store_id: "SID-1".into(),
            category: "software".into(),
            subcategory: "game".into(),
        }).await.unwrap();
    }
    let list = {
        let work_repo = repo.work();
        WorkRepository::list_all_details(work_repo).await.unwrap()
    };
    assert_eq!(list.len(), 1);
    let item = &list[0];
    assert_eq!(item.work.title, "Title A");
    let dmm = item.dmm.as_ref().expect("DMM should exist");
    assert_eq!(dmm.title, "Title A");
    assert_eq!(dmm.store_id, "SID-1");
    assert_eq!(dmm.category, "software");
    assert_eq!(dmm.subcategory, "game");
    assert!(item.dlsite.is_none());
    assert!(!item.is_dmm_omitted);
    assert!(!item.is_dlsite_omitted);
    assert!(!item.is_dmm_pack);
    assert!(item.collection_element_id.is_none());
}

