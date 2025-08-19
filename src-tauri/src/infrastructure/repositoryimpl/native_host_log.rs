use chrono::{Local, TimeZone};
use sqlx::{query, query_as};
use async_trait::async_trait;

use super::repository::RepositoryImpl;
use crate::domain::repository::native_host_log::NativeHostLogRepository;
use crate::domain::native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow};
use super::models::native_host_log::NativeHostLogTable;

#[async_trait]
impl<T: Send + Sync> NativeHostLogRepository for RepositoryImpl<T> {
    async fn insert_log(&self, level: HostLogLevel, typ: HostLogType, message: &str) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("INSERT INTO native_messaging_host_logs (level, type, message) VALUES (?, ?, ?)")
            .bind(level as i64)
            .bind(typ as i64)
            .bind(message)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn list_logs(
        &self,
        limit: i64,
        offset: i64,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<Vec<NativeHostLogRow>> {
        let pool = self.pool.0.clone();

        let mut sql = String::from("SELECT id, level, type, message, created_at FROM native_messaging_host_logs");
        let mut where_added = false;
        if level.is_some() {
            sql.push_str(" WHERE level = ?");
            where_added = true;
        }
        if typ.is_some() {
            sql.push_str(if where_added { " AND type = ?" } else { " WHERE type = ?" });
        }
        sql.push_str(" ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?");

        let mut q = query_as::<_, NativeHostLogTable>(&sql);
        if let Some(level) = level { q = q.bind(level as i64); }
        if let Some(typ) = typ { q = q.bind(typ as i64); }
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(&*pool).await?;
        let items = rows
            .into_iter()
            .map(|t| NativeHostLogRow {
                id: crate::domain::Id::new(t.id as i32),
                level: match t.level { 1 => HostLogLevel::Info, 2 => HostLogLevel::Warn, 3 => HostLogLevel::Error, _ => HostLogLevel::Info },
                r#type: match t.r#type {
                    0 => HostLogType::Unknown,
                    1 => HostLogType::ReceiveDmmSyncGamesRequest,
                    2 => HostLogType::ReceiveDlsiteSyncGamesRequest,
                    10 => HostLogType::ImageQueueWorkerStarted,
                    11 => HostLogType::ImageQueueWorkerFinished,
                    20 => HostLogType::ImageQueueItemStarted,
                    21 => HostLogType::ImageQueueItemSucceeded,
                    22 => HostLogType::ImageQueueItemFailed,
                    _ => HostLogType::Unknown,
                },
                message: t.message,
                created_at: Local.from_utc_datetime(&t.created_at),
            })
            .collect();
        Ok(items)
    }

    async fn count_logs(
        &self,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<i64> {
        let pool = self.pool.0.clone();

        let mut sql = String::from("SELECT COUNT(*) as cnt FROM native_messaging_host_logs");
        let mut where_added = false;
        if level.is_some() {
            sql.push_str(" WHERE level = ?");
            where_added = true;
        }
        if typ.is_some() {
            sql.push_str(if where_added { " AND type = ?" } else { " WHERE type = ?" });
        }
        let mut q = query_as::<_, (i64,)>(&sql);
        if let Some(level) = level { q = q.bind(level as i64); }
        if let Some(typ) = typ { q = q.bind(typ as i64); }
        let (cnt,) = q.fetch_one(&*pool).await?;
        Ok(cnt)
    }
}


