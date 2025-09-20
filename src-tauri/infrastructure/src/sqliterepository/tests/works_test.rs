use super::TestDatabase;
use domain::repository::collection::CollectionRepository;
use domain::repository::{
    works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::works::{NewDlsiteWork, NewDmmWork, NewWork};
use domain::{collection::NewCollectionElement, Id};

#[tokio::test]
async fn dmm_works_upsert_and_find_by_store_key() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = repo
        .work()
        .upsert(&NewWork {
            title: "Title A".into(),
        })
        .await
        .unwrap();
    let work_id_for_update = work_id.clone();

    let id = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-1".into(),
            category: "software".into(),
            subcategory: "game".into(),
            work_id,
        })
        .await
        .unwrap();
    assert!(id.value > 0);

    let found = repo
        .dmm_work()
        .find_by_store_key("SID-1", "software", "game")
        .await
        .unwrap();
    assert!(found.is_some());
    let w = found.unwrap();
    assert_eq!(w.store_id, "SID-1");
    assert_eq!(w.category, "software");
    assert_eq!(w.subcategory, "game");

    // 同一 store_id を別 subcategory で更新（work は同じを想定）
    let id2 = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-1".into(),
            category: "software".into(),
            subcategory: "utility".into(),
            work_id: work_id_for_update,
        })
        .await
        .unwrap();
    assert_eq!(id.value, id2.value);

    let updated = repo
        .dmm_work()
        .find_by_store_key("SID-1", "software", "utility")
        .await
        .unwrap();
    assert!(updated.is_some());
    let w2 = updated.unwrap();
    assert!(w2.work_id.value > 0);
    assert_eq!(w2.subcategory, "utility");
}

#[tokio::test]
async fn dmm_works_find_by_work_id_should_return_matching_row() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // create work and dmm row
    let work_id = repo
        .work()
        .upsert(&NewWork {
            title: "W-FIND".into(),
        })
        .await
        .unwrap();
    let _ = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-WF".into(),
            category: "game".into(),
            subcategory: "pc".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // find by work_id
    let found = repo
        .dmm_work()
        .find_by_work_id(work_id.clone())
        .await
        .unwrap();
    assert!(found.is_some());
    let row = found.unwrap();
    assert_eq!(row.work_id.value, work_id.value);
    assert_eq!(row.store_id, "SID-WF");

    // not found case
    let other = repo
        .work()
        .upsert(&NewWork {
            title: "OTHER".into(),
        })
        .await
        .unwrap();
    let none = repo.dmm_work().find_by_work_id(other).await.unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn dlsite_works_upsert_and_find_by_store_key() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = repo
        .work()
        .upsert(&NewWork {
            title: "DL Title".into(),
        })
        .await
        .unwrap();
    let work_id_for_update = work_id.clone();

    let id = repo
        .dlsite_work()
        .upsert(&NewDlsiteWork {
            store_id: "RJ123".into(),
            category: "software".into(),
            work_id,
        })
        .await
        .unwrap();
    assert!(id.value > 0);

    let found = repo
        .dlsite_work()
        .find_by_store_key("RJ123", "software")
        .await
        .unwrap();
    assert!(found.is_some());
    let w = found.unwrap();
    assert_eq!(w.store_id, "RJ123");
    assert_eq!(w.category, "software");

    // 同一 store_id を別 category で更新（work は同じを想定）
    let id2 = repo
        .dlsite_work()
        .upsert(&NewDlsiteWork {
            store_id: "RJ123".into(),
            category: "doujin".into(),
            work_id: work_id_for_update,
        })
        .await
        .unwrap();
    assert_eq!(id.value, id2.value);

    let updated = repo
        .dlsite_work()
        .find_by_store_key("RJ123", "doujin")
        .await
        .unwrap();
    assert!(updated.is_some());
    let w2 = updated.unwrap();
    assert!(w2.work_id.value > 0);
    assert_eq!(w2.category, "doujin");
}

#[tokio::test]
async fn list_all_details_dmm_only() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();
    {
        let mut dmm_repo = repo.dmm_work();
        // work を作成
        let work_id = repo
            .work()
            .upsert(&NewWork {
                title: "Title A".into(),
            })
            .await
            .unwrap();
        let _ = dmm_repo
            .upsert(&NewDmmWork {
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
                work_id,
            })
            .await
            .unwrap();
    }
    let list = {
        let mut work_repo = repo.work();
        work_repo.list_all_details().await.unwrap()
    };
    assert_eq!(list.len(), 1);
    let item = &list[0];
    assert_eq!(item.work.title, "Title A");
    let dmm = item.dmm.as_ref().expect("DMM should exist");
    assert_eq!(dmm.work_id.value, item.work.id.value);
    assert_eq!(dmm.store_id, "SID-1");
    assert_eq!(dmm.category, "software");
    assert_eq!(dmm.subcategory, "game");
    assert!(item.dlsite.is_none());
    assert!(!item.is_omitted);
    assert!(!item.is_dmm_pack);
    assert!(item.collection_element_id.is_none());
}

#[tokio::test]
async fn find_details_by_collection_element_should_match_mapping() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 1) Work と DMM を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "Title A".into(),
        })
        .await
        .unwrap()
    };
    {
        let mut r = repo.dmm_work();
        let _ = r
            .upsert(&NewDmmWork {
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
                work_id: work_id.clone(),
            })
            .await
            .unwrap();
    }

    // 2) CollectionElement を作成して Work と紐づけ
    {
        let mut c = repo.collection();
        let ce_id = Id::new(100);
        c.upsert_collection_element(&NewCollectionElement::new(ce_id.clone(), "GameName".into()))
            .await
            .unwrap();
        c.upsert_work_mapping(&ce_id, work_id.clone())
            .await
            .unwrap();
    }

    // 3) collection_element_id で WorkDetails を取得できること
    let found = {
        let mut w = repo.work();
        w.find_details_by_collection_element_id(Id::new(100))
            .await
            .unwrap()
    };
    assert!(found.is_some());
    let details = found.unwrap();
    assert_eq!(details.collection_element_id.as_ref().unwrap().value, 100);
}
