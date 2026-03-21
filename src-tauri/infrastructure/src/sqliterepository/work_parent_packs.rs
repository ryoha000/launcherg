use crate::sqliterepository::sqliterepository::RepositoryImpl;
use domain::{repository::work_parent_packs::WorkParentPacksRepository, works::Work, StrId};

impl WorkParentPacksRepository for RepositoryImpl<domain::work_parent_pack::WorkParentPack> {
    async fn add(
        &mut self,
        work_id: StrId<Work>,
        parent_pack_work_id: StrId<Work>,
    ) -> anyhow::Result<()> {
        let work_id_str = work_id.value.clone();
        let parent_pack_work_id_str = parent_pack_work_id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"INSERT OR IGNORE INTO work_parent_packs (work_id, parent_pack_work_id) VALUES (?, ?)"#,
                    )
                    .bind(work_id_str)
                    .bind(parent_pack_work_id_str)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn exists(
        &mut self,
        work_id: StrId<Work>,
        parent_pack_work_id: StrId<Work>,
    ) -> anyhow::Result<bool> {
        let work_id_str = work_id.value.clone();
        let parent_pack_work_id_str = parent_pack_work_id.value.clone();
        let row: Option<(i64,)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(
                    r#"SELECT 1 FROM work_parent_packs WHERE work_id=? AND parent_pack_work_id=? LIMIT 1"#,
                )
                .bind(work_id_str)
                .bind(parent_pack_work_id_str)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;
        Ok(row.is_some())
    }

    async fn find_parent_id(
        &mut self,
        work_id: StrId<Work>,
    ) -> anyhow::Result<Option<StrId<Work>>> {
        let wid = work_id.value.clone();
        let row: Option<(String,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<(String,)> = sqlx::query_as(
                    r#"SELECT parent_pack_work_id FROM work_parent_packs WHERE work_id=? LIMIT 1"#,
                )
                .bind(wid)
                .fetch_optional(conn)
                .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|(pid,)| domain::StrId::new(pid)))
    }
}
