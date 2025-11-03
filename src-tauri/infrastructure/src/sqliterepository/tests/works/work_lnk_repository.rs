use super::super::TestDatabase;
use domain::repository::{
    work_lnk::NewWorkLnk, work_lnk::WorkLnkRepository, works::WorkRepository, RepositoriesExt,
};
use domain::works::NewWork;

#[tokio::test]
async fn work_lnk_crud_一連の操作() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "LNK-TEST".into(),
        })
        .await
        .unwrap()
    };

    // insert
    let lnk_id1 = {
        let mut r = repo.work_lnk();
        r.insert(&NewWorkLnk {
            work_id: work_id.clone(),
            lnk_path: "C:/games/test1.lnk".into(),
        })
        .await
        .unwrap()
    };
    assert!(lnk_id1.value > 0);

    let lnk_id2 = {
        let mut r = repo.work_lnk();
        r.insert(&NewWorkLnk {
            work_id: work_id.clone(),
            lnk_path: "C:/games/test2.lnk".into(),
        })
        .await
        .unwrap()
    };
    assert!(lnk_id2.value > lnk_id1.value);

    // find_by_id
    let found = {
        let mut r = repo.work_lnk();
        r.find_by_id(lnk_id1.clone()).await.unwrap()
    };
    assert!(found.is_some());
    let lnk = found.unwrap();
    assert_eq!(lnk.id.value, lnk_id1.value);
    assert_eq!(lnk.work_id.value, work_id.value);
    assert_eq!(lnk.lnk_path, "C:/games/test1.lnk");

    // list_by_work_id（昇順）
    let list = {
        let mut r = repo.work_lnk();
        r.list_by_work_id(work_id.clone()).await.unwrap()
    };
    assert_eq!(list.len(), 2);
    assert!(list[0].id.value < list[1].id.value);
    assert_eq!(list[0].lnk_path, "C:/games/test1.lnk");
    assert_eq!(list[1].lnk_path, "C:/games/test2.lnk");

    // delete
    {
        let mut r = repo.work_lnk();
        r.delete(lnk_id1.clone()).await.unwrap();
    }

    // 再検索で消えていること
    let not_found = {
        let mut r = repo.work_lnk();
        r.find_by_id(lnk_id1).await.unwrap()
    };
    assert!(not_found.is_none());

    // list_by_work_id で1件減っていること
    let list_after_delete = {
        let mut r = repo.work_lnk();
        r.list_by_work_id(work_id).await.unwrap()
    };
    assert_eq!(list_after_delete.len(), 1);
    assert_eq!(list_after_delete[0].lnk_path, "C:/games/test2.lnk");
}
