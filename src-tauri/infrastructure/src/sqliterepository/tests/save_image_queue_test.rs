use super::TestDatabase;
use domain::repository::{RepositoriesExt, save_image_queue::ImageSaveQueueRepository};
use domain::save_image_queue::{ImagePreprocess, ImageSrcType};

#[tokio::test]
async fn save_image_queue_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // enqueue
    let id = { let mut r = repo.image_queue(); r.enqueue("http://img", ImageSrcType::Url, "dst", ImagePreprocess::None).await.unwrap() };

    // list_unfinished_oldest
    {
        let mut r = repo.image_queue();
        let rows = r.list_unfinished_oldest(10).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id.value, id.value);
    }

    // mark_failed
    {
        let mut r = repo.image_queue();
        r.mark_failed(id.clone(), "err").await.unwrap();
    }

    // mark_finished
    {
        let mut r = repo.image_queue();
        r.mark_finished(id).await.unwrap();
        let rows = r.list_unfinished_oldest(10).await.unwrap();
        assert_eq!(rows.len(), 0);
    }
}


