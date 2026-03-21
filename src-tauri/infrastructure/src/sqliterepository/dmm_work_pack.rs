use crate::sqliterepository::sqliterepository::RepositoryImpl;
use domain::{
    dmm_work_pack::DmmWorkPack, repository::dmm_work_pack::DmmPackRepository, works::Work, Id,
    StrId,
};

impl DmmPackRepository for RepositoryImpl<domain::dmm_work_pack::DmmWorkPack> {
    async fn add(&mut self, work_id: StrId<Work>) -> anyhow::Result<()> {
        let work_id_str = work_id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"INSERT OR IGNORE INTO dmm_work_packs (work_id) VALUES (?)"#)
                        .bind(work_id_str)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn remove(&mut self, work_id: StrId<Work>) -> anyhow::Result<()> {
        let work_id_str = work_id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"DELETE FROM dmm_work_packs WHERE work_id = ?"#)
                        .bind(work_id_str)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn list(&mut self) -> anyhow::Result<Vec<DmmWorkPack>> {
        let rows: Vec<(i64, String)> =
            self.executor
                .with_conn(|conn| {
                    Box::pin(async move {
                        Ok(sqlx::query_as(
                            r#"SELECT id, work_id FROM dmm_work_packs ORDER BY id DESC"#,
                        )
                        .fetch_all(conn)
                        .await?)
                    })
                })
                .await?;
        Ok(rows
            .into_iter()
            .map(|(id, work_id)| DmmWorkPack {
                id: Id::new(id as i32),
                work_id: StrId::new(work_id),
            })
            .collect())
    }

    async fn exists(&mut self, work_id: StrId<Work>) -> anyhow::Result<bool> {
        let work_id_str = work_id.value.clone();
        let row: Option<(i64,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(
                        sqlx::query_as(r#"SELECT 1 FROM dmm_work_packs WHERE work_id = ? LIMIT 1"#)
                            .bind(work_id_str)
                            .fetch_optional(conn)
                            .await?,
                    )
                })
            })
            .await?;
        Ok(row.is_some())
    }
}
