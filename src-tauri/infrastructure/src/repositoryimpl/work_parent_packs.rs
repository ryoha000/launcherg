use sqlx::query_as;
use domain::{repository::work_parent_packs::WorkParentPacksRepository, works::Work, Id};
use super::repository::RepositoryImpl;

impl WorkParentPacksRepository for RepositoryImpl<Work> {
    async fn add(&self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        let _row: (i64,) = query_as(
            r#"INSERT OR IGNORE INTO work_parent_packs (work_id, parent_pack_work_id) VALUES (?, ?) RETURNING 1"#,
        )
        .bind(work_id.value)
        .bind(parent_pack_work_id.value)
        .fetch_one(&*pool)
        .await?;
        Ok(())
    }

    async fn exists(&self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> anyhow::Result<bool> {
        let pool = self.pool.0.clone();
        let row: Option<(i64,)> = query_as(
            r#"SELECT 1 FROM work_parent_packs WHERE work_id=? AND parent_pack_work_id=? LIMIT 1"#,
        )
        .bind(work_id.value)
        .bind(parent_pack_work_id.value)
        .fetch_optional(&*pool)
        .await?;
        Ok(row.is_some())
    }
}


