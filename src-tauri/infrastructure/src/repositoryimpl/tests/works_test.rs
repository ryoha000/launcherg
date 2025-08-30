use super::TestDatabase;
use domain::repositoryv2::{
    collection::CollectionRepository,
    dmm_work_pack::DmmPackRepository,
    work_omit::WorkOmitRepository,
    works::{DmmWorkRepository, DlsiteWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::works::{NewDmmWork, NewDlsiteWork};

mod tests {
    use super::*;

    #[tokio::test]
    async fn dmm_works_upsert_and_find_by_store_key() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.repositories.dmm_work_repository();

        // insert new
        let id = repo.upsert(&NewDmmWork {
            title: "Title A".into(),
            store_id: "SID-1".into(),
            category: "software".into(),
            subcategory: "game".into(),
        }).await.unwrap();
        assert!(id.value > 0);

        // find
        let found = repo.find_by_store_key("SID-1", "software", "game").await.unwrap();
        assert!(found.is_some());
        let w = found.unwrap();
        assert_eq!(w.store_id, "SID-1");
        assert_eq!(w.category, "software");
        assert_eq!(w.subcategory, "game");

        // update same store_id
        let id2 = repo.upsert(&NewDmmWork {
            title: "Title A2".into(),
            store_id: "SID-1".into(),
            category: "software".into(),
            subcategory: "utility".into(),
        }).await.unwrap();
        assert_eq!(id.value, id2.value);

        let updated = repo.find_by_store_key("SID-1", "software", "utility").await.unwrap();
        assert!(updated.is_some());
        let w2 = updated.unwrap();
        assert_eq!(w2.title, "Title A2");
        assert_eq!(w2.subcategory, "utility");
    }

    #[tokio::test]
    async fn dlsite_works_upsert_and_find_by_store_key() {
        let test_db = TestDatabase::new().await.unwrap();
        let repo = test_db.repositories.dlsite_work_repository();

        // insert new
        let id = repo.upsert(&NewDlsiteWork {
            title: "DL Title".into(),
            store_id: "RJ123".into(),
            category: "software".into(),
        }).await.unwrap();
        assert!(id.value > 0);

        // find
        let found = repo.find_by_store_key("RJ123", "software").await.unwrap();
        assert!(found.is_some());
        let w = found.unwrap();
        assert_eq!(w.store_id, "RJ123");
        assert_eq!(w.category, "software");

        // update title/category
        let id2 = repo.upsert(&NewDlsiteWork {
            title: "DL Title 2".into(),
            store_id: "RJ123".into(),
            category: "doujin".into(),
        }).await.unwrap();
        assert_eq!(id.value, id2.value);

        let updated = repo.find_by_store_key("RJ123", "doujin").await.unwrap();
        assert!(updated.is_some());
        let w2 = updated.unwrap();
        assert_eq!(w2.title, "DL Title 2");
        assert_eq!(w2.category, "doujin");
    }

    #[tokio::test]
    async fn list_all_details_dmm_only() {
        let test_db = TestDatabase::new().await.unwrap();
        let dmm_repo = test_db.repositories.dmm_work_repository();
        let work_repo = test_db.repositories.work_repository();

        // DMM のみ登録
        let _ = dmm_repo
            .upsert(&NewDmmWork {
                title: "Title A".into(),
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
            })
            .await
            .unwrap();

        let list = work_repo.list_all_details().await.unwrap();
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

    #[tokio::test]
    async fn list_all_details_mapping_and_flags() {
        let test_db = TestDatabase::new().await.unwrap();
        let dmm_repo = test_db.repositories.dmm_work_repository();
        let work_repo = test_db.repositories.work_repository();
        let col_repo = test_db.repositories.collection_repository();
        let omit_repo = test_db.repositories.work_omit_repository();
        let pack_repo = test_db.repositories.dmm_pack_repository();

        // DMM 登録（works も作成される）
        let _ = dmm_repo
            .upsert(&NewDmmWork {
                title: "Title A".into(),
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
            })
            .await
            .unwrap();

        // Work を取得
        let work = work_repo.find_by_title("Title A").await.unwrap().unwrap();

        // コレクション要素を確保してマッピング
        let ce_id = col_repo
            .allocate_new_collection_element_id("Title A")
            .await
            .unwrap();
        col_repo
            .upsert_work_mapping(&ce_id, work.id.value)
            .await
            .unwrap();

        // omit/pack フラグを付与
        omit_repo.add(work.id.clone()).await.unwrap();
        pack_repo.add(work.id.clone()).await.unwrap();

        // 確認
        let list = work_repo.list_all_details().await.unwrap();
        assert_eq!(list.len(), 1);
        let item = &list[0];
        assert_eq!(item.work.title, "Title A");
        let dmm = item.dmm.as_ref().expect("DMM should exist");
        assert_eq!(dmm.title, "Title A");
        assert_eq!(dmm.store_id, "SID-1");
        assert_eq!(dmm.category, "software");
        assert_eq!(dmm.subcategory, "game");
        let ce = item.collection_element_id.as_ref().expect("collection_element_id should exist");
        assert_eq!(ce.value, ce_id.value);
        assert!(item.is_dmm_omitted);
        assert!(item.is_dmm_pack);
        assert!(!item.is_dlsite_omitted);
    }

    #[tokio::test]
    async fn list_all_details_merge_dmm_and_dlsite() {
        let test_db = TestDatabase::new().await.unwrap();
        let dmm_repo = test_db.repositories.dmm_work_repository();
        let dl_repo = test_db.repositories.dlsite_work_repository();
        let work_repo = test_db.repositories.work_repository();

        // DMM と DLsite を別々の Work として登録
        let _ = dmm_repo
            .upsert(&NewDmmWork {
                title: "Title A".into(),
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
            })
            .await
            .unwrap();
        let _ = dl_repo
            .upsert(&NewDlsiteWork {
                title: "DL Title".into(),
                store_id: "RJ123".into(),
                category: "software".into(),
            })
            .await
            .unwrap();

        let mut list = work_repo.list_all_details().await.unwrap();
        assert_eq!(list.len(), 2);

        // タイトルで取り出して検証
        list.sort_by(|a, b| a.work.title.cmp(&b.work.title));
        let a = &list[0];
        let b = &list[1];

        if a.work.title == "DL Title" {
            let dl = a.dlsite.as_ref().expect("DLsite should exist");
            assert_eq!(dl.title, "DL Title");
            assert_eq!(dl.store_id, "RJ123");
            assert_eq!(dl.category, "software");
            assert!(a.dmm.is_none());

            let dmm = b.dmm.as_ref().expect("DMM should exist");
            assert_eq!(dmm.title, "Title A");
            assert_eq!(dmm.store_id, "SID-1");
            assert_eq!(dmm.category, "software");
            assert_eq!(dmm.subcategory, "game");
            assert!(b.dlsite.is_none());
        } else {
            let dmm = a.dmm.as_ref().expect("DMM should exist");
            assert_eq!(dmm.title, "Title A");
            assert_eq!(dmm.store_id, "SID-1");
            assert_eq!(dmm.category, "software");
            assert_eq!(dmm.subcategory, "game");
            assert!(a.dlsite.is_none());

            let dl = b.dlsite.as_ref().expect("DLsite should exist");
            assert_eq!(dl.title, "DL Title");
            assert_eq!(dl.store_id, "RJ123");
            assert_eq!(dl.category, "software");
            assert!(b.dmm.is_none());
        }
    }
}


