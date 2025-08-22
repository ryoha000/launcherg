use sqlx::{query, query_as};

use domain::{dmm_pack::DmmPackMark, repository::dmm_pack::DmmPackRepository};

use super::repository::RepositoryImpl;

impl DmmPackRepository for RepositoryImpl<DmmPackMark> {
    async fn add(&self, store_id: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("INSERT OR IGNORE INTO dmm_pack_marks (store_id) VALUES (?)")
            .bind(store_id)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn remove(&self, store_id: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("DELETE FROM dmm_pack_marks WHERE store_id = ?")
            .bind(store_id)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<DmmPackMark>> {
        let pool = self.pool.0.clone();
        let rows: Vec<(i64, String)> = query_as(
            "SELECT id, store_id FROM dmm_pack_marks ORDER BY id DESC",
        )
        .fetch_all(&*pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, sid)| DmmPackMark { id: domain::Id::new(id as i32), store_id: sid })
            .collect())
    }

    async fn exists(&self, store_id: &str) -> anyhow::Result<bool> {
        let pool = self.pool.0.clone();
        let row: Option<(i64,)> = query_as(
            "SELECT 1 FROM dmm_pack_marks WHERE store_id = ? LIMIT 1",
        )
        .bind(store_id)
        .fetch_optional(&*pool)
        .await?;
        Ok(row.is_some())
    }
}


