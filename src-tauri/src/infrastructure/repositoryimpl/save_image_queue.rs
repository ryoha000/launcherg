use sqlx::{query, query_as};
use async_trait::async_trait;

use super::repository::RepositoryImpl;
use crate::domain::{repository::save_image_queue::{ImageSaveQueueRepository}, save_image_queue::{ImageSaveQueueRow, ImageSrcType, ImagePreprocess}, Id};

#[async_trait]
impl<T: Send + Sync> ImageSaveQueueRepository for RepositoryImpl<T> {
    async fn enqueue(&self, src: &str, src_type: ImageSrcType, dst_path: &str, preprocess: ImagePreprocess) -> anyhow::Result<Id<ImageSaveQueueRow>> {
        let pool = self.pool.0.clone();
        let rec: (i64,) = query_as("INSERT INTO save_image_queue (src, src_type, dst_path, preprocess) VALUES (?, ?, ?, ?) RETURNING id")
            .bind(src)
            .bind(match src_type { ImageSrcType::Url => 1_i64, ImageSrcType::Path => 2_i64 })
            .bind(dst_path)
            .bind(match preprocess { ImagePreprocess::None => 0_i64, ImagePreprocess::ResizeAndCropSquare256 => 1_i64, ImagePreprocess::ResizeForWidth400 => 2_i64 })
            .fetch_one(&*pool)
            .await?;
        Ok(Id::new(rec.0 as i32))
    }

    async fn list_unfinished_oldest(&self, limit: i64) -> anyhow::Result<Vec<ImageSaveQueueRow>> {
        let pool = self.pool.0.clone();
        let rows: Vec<super::models::save_image_queue::SaveImageQueueTable> = query_as(
            "SELECT id, src, src_type, dst_path, preprocess, last_error FROM save_image_queue WHERE finished_at IS NULL ORDER BY created_at ASC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&*pool)
        .await?;
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

    async fn mark_finished(&self, id: Id<ImageSaveQueueRow>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("UPDATE save_image_queue SET finished_at = CURRENT_TIMESTAMP, last_error = NULL WHERE id = ?")
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn mark_failed(&self, id: Id<ImageSaveQueueRow>, error: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("UPDATE save_image_queue SET finished_at = CURRENT_TIMESTAMP, last_error = ? WHERE id = ?")
            .bind(error)
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
}


