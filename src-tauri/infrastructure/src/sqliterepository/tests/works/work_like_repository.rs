use super::super::TestDatabase;
use chrono::{DateTime, Local};
use domain::repository::{work_like::WorkLikeRepository, works::WorkRepository, RepositoriesExt};
use domain::works::{NewWork, NewWorkLike};

#[tokio::test]
async fn work_like_upsert_get_delete_一連の操作() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "LIKE-TEST".into(),
        })
        .await
        .unwrap()
    };

    // upsert（初回挿入）
    let like_at1 = DateTime::parse_from_rfc3339("2024-01-15T10:00:00+09:00")
        .unwrap()
        .with_timezone(&Local);
    let like_id = {
        let mut r = repo.work_like();
        r.upsert(&NewWorkLike {
            work_id: work_id.clone(),
            like_at: like_at1,
        })
        .await
        .unwrap()
    };
    assert!(like_id.value > 0);

    // get_by_work_id で取得
    let found = {
        let mut r = repo.work_like();
        r.get_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(found.is_some());
    let like = found.unwrap();
    assert_eq!(like.id.value, like_id.value);
    assert_eq!(like.work_id.value, work_id.value);
    assert_eq!(like.like_at, like_at1);

    // upsert（更新: ON CONFLICT）
    let like_at2 = DateTime::parse_from_rfc3339("2024-01-20T15:30:00+09:00")
        .unwrap()
        .with_timezone(&Local);
    let like_id2 = {
        let mut r = repo.work_like();
        r.upsert(&NewWorkLike {
            work_id: work_id.clone(),
            like_at: like_at2,
        })
        .await
        .unwrap()
    };
    // 同じIDが返る（更新された）
    assert_eq!(like_id.value, like_id2.value);

    // 更新されたことを確認
    let updated = {
        let mut r = repo.work_like();
        r.get_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(updated.is_some());
    let like2 = updated.unwrap();
    assert_eq!(like2.like_at, like_at2);

    // delete_by_work_id
    {
        let mut r = repo.work_like();
        r.delete_by_work_id(work_id.clone()).await.unwrap();
    }

    // 削除されたことを確認
    let deleted = {
        let mut r = repo.work_like();
        r.get_by_work_id(work_id).await.unwrap()
    };
    assert!(deleted.is_none());
}

#[tokio::test]
async fn work_like_update_like_at_by_work_id_分岐() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "LIKE-UPDATE".into(),
        })
        .await
        .unwrap()
    };

    // ケース1: Some(like_at) → upsert
    let like_at = DateTime::parse_from_rfc3339("2024-01-25T12:00:00+09:00")
        .unwrap()
        .with_timezone(&Local);
    {
        let mut r = repo.work_like();
        r.update_like_at_by_work_id(work_id.clone(), Some(like_at))
            .await
            .unwrap();
    }

    // 挿入されたことを確認
    let found = {
        let mut r = repo.work_like();
        r.get_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(found.is_some());
    let like = found.unwrap();
    assert_eq!(like.like_at, like_at);

    // ケース2: None → delete
    {
        let mut r = repo.work_like();
        r.update_like_at_by_work_id(work_id.clone(), None)
            .await
            .unwrap();
    }

    // 削除されたことを確認
    let deleted = {
        let mut r = repo.work_like();
        r.get_by_work_id(work_id).await.unwrap()
    };
    assert!(deleted.is_none());
}
