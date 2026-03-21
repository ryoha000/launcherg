use super::super::TestDatabase;
use domain::repository::{
    works::{DlsiteWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::works::{NewDlsiteWork, NewWork};

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

    let found_by_store_id = repo.dlsite_work().find_by_store_id("RJ123").await.unwrap();
    assert!(found_by_store_id.is_some());
    assert_eq!(found_by_store_id.unwrap().category, "software");

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
    assert!(!w2.work_id.value.is_empty());
    assert_eq!(w2.category, "doujin");

    let updated_by_store_id = repo.dlsite_work().find_by_store_id("RJ123").await.unwrap();
    assert!(updated_by_store_id.is_some());
    assert_eq!(updated_by_store_id.unwrap().category, "doujin");
}
