use super::super::TestDatabase;
use domain::repository::{
    works::{DmmWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::works::{NewDmmWork, NewWork};

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

    let found_by_store_id = repo.dmm_work().find_by_store_id("SID-1").await.unwrap();
    assert!(found_by_store_id.is_some());
    assert_eq!(found_by_store_id.unwrap().store_id, "SID-1");

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
    assert!(!w2.work_id.value.is_empty());
    assert_eq!(w2.subcategory, "utility");

    let updated_by_store_id = repo.dmm_work().find_by_store_id("SID-1").await.unwrap();
    assert!(updated_by_store_id.is_some());
    assert_eq!(updated_by_store_id.unwrap().subcategory, "utility");
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
async fn dmm_find_by_store_keys_空_単一_複数() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id1 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "DMM-K1".into(),
        })
        .await
        .unwrap()
    };
    let work_id2 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "DMM-K2".into(),
        })
        .await
        .unwrap()
    };
    let work_id3 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "DMM-K3".into(),
        })
        .await
        .unwrap()
    };

    // DMM works を追加
    let _id1 = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-K1".into(),
            category: "software".into(),
            subcategory: "game".into(),
            work_id: work_id1.clone(),
        })
        .await
        .unwrap();

    let _id2 = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-K2".into(),
            category: "software".into(),
            subcategory: "utility".into(),
            work_id: work_id2.clone(),
        })
        .await
        .unwrap();

    let _id3 = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-K3".into(),
            category: "doujin".into(),
            subcategory: "game".into(),
            work_id: work_id3.clone(),
        })
        .await
        .unwrap();

    // ケース1: 空配列
    let empty_result = {
        let mut r = repo.dmm_work();
        r.find_by_store_keys(&[]).await.unwrap()
    };
    assert!(empty_result.is_empty());

    // ケース2: 単一キー（存在）
    let single_result = {
        let mut r = repo.dmm_work();
        r.find_by_store_keys(&[(
            "SID-K1".to_string(),
            "software".to_string(),
            "game".to_string(),
        )])
        .await
        .unwrap()
    };
    assert_eq!(single_result.len(), 1);
    assert_eq!(single_result[0].store_id, "SID-K1");
    assert_eq!(single_result[0].work_id.value, work_id1.value);

    // ケース3: 複数キー（すべて存在）
    let multiple_result = {
        let mut r = repo.dmm_work();
        r.find_by_store_keys(&[
            (
                "SID-K1".to_string(),
                "software".to_string(),
                "game".to_string(),
            ),
            (
                "SID-K2".to_string(),
                "software".to_string(),
                "utility".to_string(),
            ),
            (
                "SID-K3".to_string(),
                "doujin".to_string(),
                "game".to_string(),
            ),
        ])
        .await
        .unwrap()
    };
    assert_eq!(multiple_result.len(), 3);
    let mut result_store_ids: Vec<String> =
        multiple_result.iter().map(|w| w.store_id.clone()).collect();
    result_store_ids.sort();
    assert_eq!(result_store_ids, vec!["SID-K1", "SID-K2", "SID-K3"]);

    // ケース4: 存在するキーと存在しないキーの混在
    let mixed_result = {
        let mut r = repo.dmm_work();
        r.find_by_store_keys(&[
            (
                "SID-K1".to_string(),
                "software".to_string(),
                "game".to_string(),
            ),
            (
                "SID-NOTEXIST".to_string(),
                "software".to_string(),
                "game".to_string(),
            ),
            (
                "SID-K2".to_string(),
                "software".to_string(),
                "utility".to_string(),
            ),
        ])
        .await
        .unwrap()
    };
    // 存在する分のみ返る
    assert_eq!(mixed_result.len(), 2);
    let mut mixed_store_ids: Vec<String> =
        mixed_result.iter().map(|w| w.store_id.clone()).collect();
    mixed_store_ids.sort();
    assert_eq!(mixed_store_ids, vec!["SID-K1", "SID-K2"]);
}
