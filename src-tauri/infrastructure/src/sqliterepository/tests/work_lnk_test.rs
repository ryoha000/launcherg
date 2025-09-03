use super::TestDatabase;
use domain::repository::{RepositoriesExt, work_lnk::WorkLnkRepository, works::WorkRepository};
use domain::repository::work_lnk::NewWorkLnk;
use domain::works::NewWork;

#[tokio::test]
async fn work_lnk_insert_list_delete_flow() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = { let mut r = repo.work(); r.upsert(&NewWork { title: "W".into() }).await.unwrap() };

    // insert 2 rows
    {
        let mut r = repo.work_lnk();
        let id1 = r.insert(&NewWorkLnk { work_id: work_id.clone(), lnk_path: "C:/games/W1.lnk".into() }).await.unwrap();
        assert!(id1.value > 0);
        let id2 = r.insert(&NewWorkLnk { work_id: work_id.clone(), lnk_path: "C:/games/W2.lnk".into() }).await.unwrap();
        assert!(id2.value > id1.value);
    }

    // list_by_work_id should return both in ascending id order
    {
        let mut r = repo.work_lnk();
        let list = r.list_by_work_id(work_id.clone()).await.unwrap();
        assert_eq!(list.len(), 2);
        assert!(list[0].id.value < list[1].id.value);
        assert_eq!(list[0].work_id.value, work_id.value);
        assert_eq!(list[1].work_id.value, work_id.value);
        assert_eq!(list[0].lnk_path, "C:/games/W1.lnk");
        assert_eq!(list[1].lnk_path, "C:/games/W2.lnk");
    }

    // delete one and list again
    {
        let mut r = repo.work_lnk();
        let list = r.list_by_work_id(work_id.clone()).await.unwrap();
        let delete_id = list[0].id.clone();
        r.delete(delete_id).await.unwrap();
        let list2 = r.list_by_work_id(work_id.clone()).await.unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0].lnk_path, "C:/games/W2.lnk");
    }
}

#[tokio::test]
async fn work_lnk_unique_lnk_path_violation() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id1 = { let mut r = repo.work(); r.upsert(&NewWork { title: "W1".into() }).await.unwrap() };
    let work_id2 = { let mut r = repo.work(); r.upsert(&NewWork { title: "W2".into() }).await.unwrap() };

    // 同一 lnk_path の重複は UNIQUE 制約で失敗する
    let res = {
        let mut r = repo.work_lnk();
        let _ = r.insert(&NewWorkLnk { work_id: work_id1.clone(), lnk_path: "C:/games/DUP.lnk".into() }).await.unwrap();
        r.insert(&NewWorkLnk { work_id: work_id2.clone(), lnk_path: "C:/games/DUP.lnk".into() }).await
    };

    // エラーが返ることだけを確認（詳細メッセージまでは固定しない）
    assert!(res.is_err());
}

#[tokio::test]
async fn work_lnk_empty_list_when_no_records() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = { let mut r = repo.work(); r.upsert(&NewWork { title: "W".into() }).await.unwrap() };

    // まだ挿入していないので空が返る
    let list = { let mut r = repo.work_lnk(); r.list_by_work_id(work_id).await.unwrap() };
    assert!(list.is_empty());
}


#[tokio::test]
async fn work_lnk_find_by_id_success() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = { let mut r = repo.work(); r.upsert(&NewWork { title: "W".into() }).await.unwrap() };

    // 1件挿入して、その id で取得できること
    let inserted_id = {
        let mut r = repo.work_lnk();
        r.insert(&NewWorkLnk { work_id: work_id.clone(), lnk_path: "C:/games/W1.lnk".into() }).await.unwrap()
    };

    let found = { let mut r = repo.work_lnk(); r.find_by_id(inserted_id.clone()).await.unwrap() };
    assert!(found.is_some());
    let row = found.unwrap();
    assert_eq!(row.id.value, inserted_id.value);
    assert_eq!(row.work_id.value, work_id.value);
    assert_eq!(row.lnk_path, "C:/games/W1.lnk");
}

#[tokio::test]
async fn work_lnk_find_by_id_not_found() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 存在しない id を検索すると None
    let not_found = { let mut r = repo.work_lnk(); r.find_by_id(domain::Id::new(999999)).await.unwrap() };
    assert!(not_found.is_none());
}


