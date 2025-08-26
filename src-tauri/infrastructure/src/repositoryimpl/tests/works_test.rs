use super::TestDatabase;
use domain::repository::{RepositoriesExt, works::{DmmWorkRepository, DlsiteWorkRepository}};
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
}


