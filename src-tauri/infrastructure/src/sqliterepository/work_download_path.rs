use domain::{repository::work_download_path::WorkDownloadPathRepository, work_download_path::WorkDownloadPath, works::Work, Id};
use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl WorkDownloadPathRepository for RepositoryImpl<domain::work_download_path::WorkDownloadPath> {
    async fn add(&mut self, work_id: Id<Work>, download_path: &str) -> anyhow::Result<()> {
        let download_path = download_path.to_string();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query(r#"INSERT INTO work_download_paths (work_id, download_path) VALUES (?, ?)"#)
                    .bind(work_id.value)
                    .bind(download_path)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn list_by_work(&mut self, work_id: Id<Work>) -> anyhow::Result<Vec<WorkDownloadPath>> {
        let rows: Vec<(i64, i64, String)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(r#"SELECT id, work_id, download_path FROM work_download_paths WHERE work_id = ? ORDER BY id DESC"#)
                    .bind(work_id.value)
                    .fetch_all(conn)
                    .await?)
            })
        }).await?;
        Ok(rows.into_iter().map(|(id, work_id, download_path)| WorkDownloadPath { id: Id::new(id as i32), work_id: Id::new(work_id as i32), download_path }).collect())
    }

    async fn latest_by_work(&mut self, work_id: Id<Work>) -> anyhow::Result<Option<WorkDownloadPath>> {
        let row: Option<(i64, i64, String)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(r#"SELECT id, work_id, download_path FROM work_download_paths WHERE work_id = ? ORDER BY id DESC LIMIT 1"#)
                    .bind(work_id.value)
                    .fetch_optional(conn)
                    .await?)
            })
        }).await?;
        Ok(row.map(|(id, work_id, download_path)| WorkDownloadPath { id: Id::new(id as i32), work_id: Id::new(work_id as i32), download_path }))
    }
}


