use sqlx::{query, query_as};

use domain::{
    work_omit::WorkOmit,
    repository::work_omit::WorkOmitRepository,
    works::Work,
    Id,
};

use super::repository::RepositoryImpl;

impl WorkOmitRepository for RepositoryImpl<WorkOmit> {
    async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(r#"INSERT OR IGNORE INTO work_omits (work_id) VALUES (?)"#)
        .bind(work_id.value)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(r#"DELETE FROM work_omits WHERE work_id = ?"#)
        .bind(work_id.value)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<WorkOmit>> {
        let pool = self.pool.0.clone();
        let rows: Vec<(i64, i64)> = query_as(
            r#"SELECT o.id, o.work_id FROM work_omits o ORDER BY o.id DESC"#,
        )
        .fetch_all(&*pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, work_id)| WorkOmit { id: domain::Id::new(id as i32), work_id: domain::Id::new(work_id as i32) })
            .collect())
    }

    async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> {
        let pool = self.pool.0.clone();
        let row: Option<(i64,)> = query_as(r#"SELECT 1 FROM work_omits WHERE work_id = ? LIMIT 1"#)
        .bind(work_id.value)
        .fetch_optional(&*pool)
        .await?;
        Ok(row.is_some())
    }
}
