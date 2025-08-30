use domain::{repository::work_omit::WorkOmitRepository, work_omit::WorkOmit, works::Work, Id};
use crate::sqliterepository::sqliterepository::SqliteRepository;

impl<'a> WorkOmitRepository for SqliteRepository<'a> {
    async fn add(&mut self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query(r#"INSERT OR IGNORE INTO work_omits (work_id) VALUES (?)"#)
                    .bind(work_id.value)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn remove(&mut self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query(r#"DELETE FROM work_omits WHERE work_id = ?"#)
                    .bind(work_id.value)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn list(&mut self) -> anyhow::Result<Vec<WorkOmit>> {
        let rows: Vec<(i64, i64)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(r#"SELECT o.id, o.work_id FROM work_omits o ORDER BY o.id DESC"#).fetch_all(conn).await?) })
        }).await?;
        Ok(rows.into_iter().map(|(id, work_id)| WorkOmit { id: Id::new(id as i32), work_id: Id::new(work_id as i32) }).collect())
    }

    async fn exists(&mut self, work_id: Id<Work>) -> anyhow::Result<bool> {
        let row: Option<(i64,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(r#"SELECT 1 FROM work_omits WHERE work_id = ? LIMIT 1"#).bind(work_id.value).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.is_some())
    }
}


