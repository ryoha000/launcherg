use super::TestDatabase;
use domain::repository::{RepositoriesExt, collection::CollectionRepository, works::WorkRepository};
use domain::collection::{NewCollectionElement, NewCollectionElementInfo, NewCollectionElementInstall, NewCollectionElementLike, NewCollectionElementPaths, NewCollectionElementPlay, NewCollectionElementThumbnail};
use domain::Id;
use domain::works::NewWork;

#[tokio::test]
async fn collection_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // upsert element id 1
    {
        let mut r = repo.collection();
        r.upsert_collection_element(&NewCollectionElement::new(Id::new(1), "G".into())).await.unwrap();
    }

    // get id allocation (should return 2)
    let new_id = { let mut r = repo.collection(); r.allocate_new_collection_element_id("H").await.unwrap() };
    assert_eq!(new_id.value, 2);

    // upsert details for id 1
    {
        let mut r = repo.collection();
        r.upsert_collection_element_info(&NewCollectionElementInfo::new(Id::new(1), "gr".into(), "b".into(), "br".into(), String::new(), false)).await.unwrap();
        r.upsert_collection_element_paths(&NewCollectionElementPaths::new(Id::new(1), Some("exe".into()), Some("lnk".into()))).await.unwrap();
        r.upsert_collection_element_install(&NewCollectionElementInstall::new(Id::new(1), chrono::Local::now())).await.unwrap();
        r.upsert_collection_element_play(&NewCollectionElementPlay::new(Id::new(1), chrono::Local::now())).await.unwrap();
        r.upsert_collection_element_like(&NewCollectionElementLike::new(Id::new(1), chrono::Local::now())).await.unwrap();
        r.upsert_collection_element_thumbnail(&NewCollectionElementThumbnail::new(Id::new(1), Some(10), Some(20))).await.unwrap();
    }

    // getters
    {
        let mut r = repo.collection();
        let one = r.get_element_by_element_id(&Id::new(1)).await.unwrap().unwrap();
        assert_eq!(one.gamename, "G");
        assert!(r.get_element_info_by_element_id(&Id::new(1)).await.unwrap().is_some());
        assert!(r.get_element_paths_by_element_id(&Id::new(1)).await.unwrap().is_some());
        assert!(r.get_element_install_by_element_id(&Id::new(1)).await.unwrap().is_some());
        assert!(r.get_element_play_by_element_id(&Id::new(1)).await.unwrap().is_some());
        assert!(r.get_element_like_by_element_id(&Id::new(1)).await.unwrap().is_some());
        assert!(r.get_element_thumbnail_by_element_id(&Id::new(1)).await.unwrap().is_some());
    }

    // list all elements
    {
        let mut r = repo.collection();
        let list = r.get_all_elements().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    // get null thumbnail size ids (id=2 has no sizes yet -> one record)
    {
        let mut r = repo.collection();
        let list = r.get_null_thumbnail_size_element_ids().await.unwrap();
        assert!(!list.is_empty());
    }

    // like/unlike
    {
        let mut r = repo.collection();
        r.update_element_like_at_by_id(&Id::new(1), None).await.unwrap();
        assert!(r.get_element_like_by_element_id(&Id::new(1)).await.unwrap().is_none());
    }

    // delete element
    {
        let mut r = repo.collection();
        r.delete_collection_element(&Id::new(1)).await.unwrap();
        assert!(r.get_element_by_element_id(&Id::new(1)).await.unwrap().is_none());
    }
}

#[tokio::test]
async fn get_work_ids_by_collection_ids_returns_mapped_pairs() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 準備: work と collection_element を作成し、マッピングを張る
    let (work_id1, work_id2) = {
        let mut w = repo.work();
        let id1 = w.upsert(&NewWork { title: "W1".into() }).await.unwrap();
        let id2 = w.upsert(&NewWork { title: "W2".into() }).await.unwrap();
        (id1, id2)
    };

    // collection elements 作成 (IDは任意の整数)
    {
        let mut c = repo.collection();
        c.upsert_collection_element(&NewCollectionElement::new(Id::new(100), "G100".into())).await.unwrap();
        c.upsert_collection_element(&NewCollectionElement::new(Id::new(200), "G200".into())).await.unwrap();
        // マッピング
        c.upsert_work_mapping(&Id::new(100), work_id1.clone()).await.unwrap();
        c.upsert_work_mapping(&Id::new(200), work_id2.clone()).await.unwrap();
    }

    // 実行: 既存/未存在混在の入力で問い合わせ
    let got = {
        let mut c = repo.collection();
        c.get_work_ids_by_collection_ids(&[Id::new(100), Id::new(200), Id::new(300)]) // 300 は未マッピング
            .await
            .unwrap()
    };

    // 検証: 100,200 のみ返る。順序はクエリ結果順（IN句に準拠しない可能性）なので集合比較
    let mut got_sorted = got.clone();
    got_sorted.sort_by_key(|(ce, _)| ce.value);

    assert_eq!(got_sorted.len(), 2);
    assert_eq!(got_sorted[0].0.value, 100);
    assert_eq!(got_sorted[0].1.value, work_id1.value);
    assert_eq!(got_sorted[1].0.value, 200);
    assert_eq!(got_sorted[1].1.value, work_id2.value);
}


