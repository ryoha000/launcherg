use domain::{repository::work_parent_packs::WorkParentPacksRepository, works::Work, Id};
use crate::sqliterepository::sqliterepository::SqliteRepository;

impl<'a> WorkParentPacksRepository for SqliteRepository<'a> {
    async fn add(&mut self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> anyhow::Result<()> {
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                let _row: (i64,) = sqlx::query_as(
                    r#"INSERT OR IGNORE INTO work_parent_packs (work_id, parent_pack_work_id) VALUES (?, ?) RETURNING 1"#,
                )
                .bind(work_id.value)
                .bind(parent_pack_work_id.value)
                .fetch_one(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn exists(&mut self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> anyhow::Result<bool> {
        let row: Option<(i64,)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(
                    r#"SELECT 1 FROM work_parent_packs WHERE work_id=? AND parent_pack_work_id=? LIMIT 1"#,
                )
                .bind(work_id.value)
                .bind(parent_pack_work_id.value)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;
        Ok(row.is_some())
    }
}


