use crate::sqliterepository::sqliterepository::RepositoryImpl;
use domain::{repository::work_parent_packs::WorkParentPacksRepository, works::Work, Id};

impl WorkParentPacksRepository for RepositoryImpl<domain::work_parent_pack::WorkParentPack> {
    async fn add(
        &mut self,
        work_id: Id<Work>,
        parent_pack_work_id: Id<Work>,
    ) -> anyhow::Result<()> {
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

    async fn exists(
        &mut self,
        work_id: Id<Work>,
        parent_pack_work_id: Id<Work>,
    ) -> anyhow::Result<bool> {
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

    async fn find_parent_id(&mut self, work_id: Id<Work>) -> anyhow::Result<Option<Id<Work>>> {
        let wid = work_id.value as i64;
        let row: Option<(i64,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<(i64,)> = sqlx::query_as(
                    r#"SELECT parent_pack_work_id FROM work_parent_packs WHERE work_id=? LIMIT 1"#,
                )
                .bind(wid)
                .fetch_optional(conn)
                .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|(pid,)| domain::Id::new(pid as i32)))
    }
}
