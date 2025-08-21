use sqlx::{query, query_as};

use domain::{deny_list::{DenyListEntry, StoreType}, repository::deny_list::DenyListRepository};

use super::repository::RepositoryImpl;

impl DenyListRepository for RepositoryImpl<DenyListEntry> {
    async fn add(&self, store_type: StoreType, store_id: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("INSERT OR IGNORE INTO denied_store_ids (store_type, store_id) VALUES (?, ?)")
            .bind(i32::from(store_type) as i64)
            .bind(store_id)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn remove(&self, store_type: StoreType, store_id: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("DELETE FROM denied_store_ids WHERE store_type = ? AND store_id = ?")
            .bind(i32::from(store_type) as i64)
            .bind(store_id)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<DenyListEntry>> {
        let pool = self.pool.0.clone();
        let rows: Vec<(i64, i64, String)> = query_as(
            "SELECT id, store_type, store_id FROM denied_store_ids ORDER BY id DESC",
        )
        .fetch_all(&*pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|(id, typ, sid)| {
                if let Ok(st) = StoreType::try_from(typ as i32) {
                    Some(DenyListEntry { id: domain::Id::new(id as i32), store_type: st, store_id: sid })
                } else { None }
            })
            .collect())
    }

    async fn exists(&self, store_type: StoreType, store_id: &str) -> anyhow::Result<bool> {
        let pool = self.pool.0.clone();
        let row: Option<(i64,)> = query_as(
            "SELECT 1 FROM denied_store_ids WHERE store_type = ? AND store_id = ? LIMIT 1",
        )
        .bind(i32::from(store_type) as i64)
        .bind(store_id)
        .fetch_optional(&*pool)
        .await?;
        Ok(row.is_some())
    }
}


