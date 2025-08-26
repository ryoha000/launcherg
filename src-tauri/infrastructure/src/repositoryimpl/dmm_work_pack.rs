 use sqlx::{query, query_as};

use domain::{dmm_work_pack::DmmWorkPack, repository::dmm_work_pack::DmmPackRepository, works::Work, Id};

use super::repository::RepositoryImpl;

impl DmmPackRepository for RepositoryImpl<DmmWorkPack> {
    async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(r#"INSERT OR IGNORE INTO dmm_work_packs (work_id) VALUES (?)"#)
            .bind(work_id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(r#"DELETE FROM dmm_work_packs WHERE work_id = ?"#)
            .bind(work_id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<DmmWorkPack>> {
        let pool = self.pool.0.clone();
        let rows: Vec<(i64, i64)> = query_as(
            r#"SELECT id, work_id FROM dmm_work_packs ORDER BY id DESC"#,
        )
        .fetch_all(&*pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, work_id)| DmmWorkPack { id: domain::Id::new(id as i32), work_id: domain::Id::new(work_id as i32) })
            .collect())
    }

    async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> {
        let pool = self.pool.0.clone();
        let row: Option<(i64,)> = query_as(r#"SELECT 1 FROM dmm_work_packs WHERE work_id = ? LIMIT 1"#)
            .bind(work_id.value)
            .fetch_optional(&*pool)
            .await?;
        Ok(row.is_some())
    }
}


