use crate::sqliterepository::sqliterepository::RepositoryImpl;
use domain::{repository::work_omit::WorkOmitRepository, work_omit::WorkOmit, works::Work, Id, StrId};

impl WorkOmitRepository for RepositoryImpl<domain::work_omit::WorkOmit> {
    async fn add(&mut self, work_id: StrId<Work>) -> anyhow::Result<()> {
        let work_id_str = work_id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"INSERT OR IGNORE INTO work_omits (work_id) VALUES (?)"#)
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
                    sqlx::query(r#"DELETE FROM work_omits WHERE work_id = ?"#)
                        .bind(work_id_str)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn list(&mut self) -> anyhow::Result<Vec<WorkOmit>> {
        let rows: Vec<(i64, String)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(sqlx::query_as(
                        r#"SELECT o.id, o.work_id FROM work_omits o ORDER BY o.id DESC"#,
                    )
                    .fetch_all(conn)
                    .await?)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|(id, work_id)| WorkOmit {
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
                        sqlx::query_as(r#"SELECT 1 FROM work_omits WHERE work_id = ? LIMIT 1"#)
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
