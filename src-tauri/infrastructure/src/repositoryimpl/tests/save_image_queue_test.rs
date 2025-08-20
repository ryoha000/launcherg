use crate::domain::repository::save_image_queue::ImageSaveQueueRepository;
use crate::domain::save_image_queue::{ImageSrcType, ImagePreprocess};
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;
use super::TestDatabase;

#[tokio::test]
async fn save_image_queue_enqueue_and_finish() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.repositories.image_queue_repository();

    // enqueue
    let id = repo.enqueue("https://example.com/image.png", ImageSrcType::Url, "/tmp/dst.png", ImagePreprocess::ResizeAndCropSquare256).await.unwrap();
    assert!(id.value > 0);

    // list
    let items = repo.list_unfinished_oldest(10).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id.value, id.value);
    assert_eq!(items[0].dst_path, "/tmp/dst.png");

    // mark finished
    repo.mark_finished(id).await.unwrap();
    let items2 = repo.list_unfinished_oldest(10).await.unwrap();
    assert!(items2.is_empty());
}

#[tokio::test]
async fn save_image_queue_mark_failed() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.repositories.image_queue_repository();

    let id = repo.enqueue("/path/to/src.png", ImageSrcType::Path, "/tmp/dst2.png", ImagePreprocess::None).await.unwrap();
    repo.mark_failed(id, "error message").await.unwrap();

    // 失敗済みはunfinishedに出ない
    let items = repo.list_unfinished_oldest(10).await.unwrap();
    assert!(items.is_empty());
}


