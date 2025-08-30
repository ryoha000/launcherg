use domain::{repository::dmm_work_pack::DmmPackRepository, dmm_work_pack::DmmWorkPack, works::Work, Id};
use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl DmmPackRepository for RepositoryImpl<domain::dmm_work_pack::DmmWorkPack> {
    async fn add(&mut self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query(r#"INSERT OR IGNORE INTO dmm_work_packs (work_id) VALUES (?)"#)
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
                sqlx::query(r#"DELETE FROM dmm_work_packs WHERE work_id = ?"#)
                    .bind(work_id.value)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn list(&mut self) -> anyhow::Result<Vec<DmmWorkPack>> {
        let rows: Vec<(i64, i64)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(r#"SELECT id, work_id FROM dmm_work_packs ORDER BY id DESC"#).fetch_all(conn).await?) })
        }).await?;
        Ok(rows.into_iter().map(|(id, work_id)| DmmWorkPack { id: Id::new(id as i32), work_id: Id::new(work_id as i32) }).collect())
    }

    async fn exists(&mut self, work_id: Id<Work>) -> anyhow::Result<bool> {
        let row: Option<(i64,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(r#"SELECT 1 FROM dmm_work_packs WHERE work_id = ? LIMIT 1"#).bind(work_id.value).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.is_some())
    }
}


