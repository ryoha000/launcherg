use domain::{repositoryv2::save_image_queue::ImageSaveQueueRepository, save_image_queue::{ImageSaveQueueRow, ImageSrcType, ImagePreprocess}, Id};
use crate::sqliterepository::models::save_image_queue::SaveImageQueueTable;
use crate::sqliterepository::sqliterepository::SqliteRepository;

impl<'a> ImageSaveQueueRepository for SqliteRepository<'a> {
    async fn enqueue(&mut self, src: &str, src_type: ImageSrcType, dst_path: &str, preprocess: ImagePreprocess) -> anyhow::Result<Id<ImageSaveQueueRow>> {
        let src = src.to_string();
        let dst_path = dst_path.to_string();
        let (id,): (i64,) = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rec: (i64,) = sqlx::query_as("INSERT INTO save_image_queue (src, src_type, dst_path, preprocess) VALUES (?, ?, ?, ?) RETURNING id")
                    .bind(src)
                    .bind(match src_type { ImageSrcType::Url => 1_i64, ImageSrcType::Path => 2_i64 })
                    .bind(dst_path)
                    .bind(match preprocess { ImagePreprocess::None => 0_i64, ImagePreprocess::ResizeAndCropSquare256 => 1_i64, ImagePreprocess::ResizeForWidth400 => 2_i64 })
                    .fetch_one(conn)
                    .await?;
                Ok::<(i64,), anyhow::Error>(rec)
            })
        }).await?;
        Ok(Id::new(id as i32))
    }

    async fn list_unfinished_oldest(&mut self, limit: i64) -> anyhow::Result<Vec<ImageSaveQueueRow>> {
        let rows: Vec<SaveImageQueueTable> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<SaveImageQueueTable> = sqlx::query_as(
                    "SELECT id, src, src_type, dst_path, preprocess, last_error FROM save_image_queue WHERE finished_at IS NULL ORDER BY created_at ASC LIMIT ?"
                )
                .bind(limit)
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;
        let mapped = rows.into_iter().map(|t| ImageSaveQueueRow {
            id: Id::new(t.id as i32),
            src: t.src,
            src_type: if t.src_type == 1 { ImageSrcType::Url } else { ImageSrcType::Path },
            dst_path: t.dst_path,
            preprocess: match t.preprocess { 0 => ImagePreprocess::None, 1 => ImagePreprocess::ResizeAndCropSquare256, _ => ImagePreprocess::ResizeForWidth400 },
            last_error: t.last_error,
        }).collect();
        Ok(mapped)
    }

    async fn mark_finished(&mut self, id: Id<ImageSaveQueueRow>) -> anyhow::Result<()> {
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query("UPDATE save_image_queue SET finished_at = CURRENT_TIMESTAMP, last_error = NULL WHERE id = ?")
                    .bind(id.value)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn mark_failed(&mut self, id: Id<ImageSaveQueueRow>, error: &str) -> anyhow::Result<()> {
        let error = error.to_string();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query("UPDATE save_image_queue SET finished_at = CURRENT_TIMESTAMP, last_error = ? WHERE id = ?")
                    .bind(error)
                    .bind(id.value)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }
}


