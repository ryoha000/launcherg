use domain::repositoryv2::native_host_log::NativeHostLogRepository;
use domain::native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow};
use domain::Id;
use crate::sqliterepository::models::native_host_log::NativeHostLogTable;
use sqlx::Row;
use chrono::TimeZone;
use crate::sqliterepository::sqliterepository::SqliteRepository;

impl<'a> NativeHostLogRepository for SqliteRepository<'a> {
    async fn insert_log(&mut self, level: HostLogLevel, typ: HostLogType, message: &str) -> anyhow::Result<()> {
        let message = message.to_string();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query("INSERT INTO native_messaging_host_logs (level, type, message) VALUES (?, ?, ?)")
                    .bind(level as i64)
                    .bind(typ as i64)
                    .bind(message)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn list_logs(&mut self, limit: i64, offset: i64, level: Option<HostLogLevel>, typ: Option<HostLogType>) -> anyhow::Result<Vec<NativeHostLogRow>> {
        let rows: Vec<NativeHostLogTable> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut sql = String::from("SELECT id, level, type, message, created_at FROM native_messaging_host_logs");
                let mut where_added = false;
                if level.is_some() { sql.push_str(" WHERE level = ?"); where_added = true; }
                if typ.is_some() { sql.push_str(if where_added { " AND type = ?" } else { " WHERE type = ?" }); }
                sql.push_str(" ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?");
                let mut q = sqlx::query_as::<_, NativeHostLogTable>(&sql);
                if let Some(level) = level { q = q.bind(level as i64); }
                if let Some(typ) = typ { q = q.bind(typ as i64); }
                q = q.bind(limit).bind(offset);
                let rows = q.fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|t| NativeHostLogRow {
            id: Id::new(t.id as i32),
            level: match t.level { 1 => HostLogLevel::Info, 2 => HostLogLevel::Warn, 3 => HostLogLevel::Error, _ => HostLogLevel::Info },
            r#type: match t.r#type { 0 => HostLogType::Unknown, 1 => HostLogType::ReceiveDmmSyncGamesRequest, 2 => HostLogType::ReceiveDlsiteSyncGamesRequest, 10 => HostLogType::ImageQueueWorkerStarted, 11 => HostLogType::ImageQueueWorkerFinished, 20 => HostLogType::ImageQueueItemStarted, 21 => HostLogType::ImageQueueItemSucceeded, 22 => HostLogType::ImageQueueItemFailed, _ => HostLogType::Unknown },
            message: t.message,
            created_at: chrono::Local.from_utc_datetime(&t.created_at),
        }).collect())
    }

    async fn count_logs(&mut self, level: Option<HostLogLevel>, typ: Option<HostLogType>) -> anyhow::Result<i64> {
        let (cnt,): (i64,) = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut sql = String::from("SELECT COUNT(*) as cnt FROM native_messaging_host_logs");
                let mut where_added = false;
                if level.is_some() { sql.push_str(" WHERE level = ?"); where_added = true; }
                if typ.is_some() { sql.push_str(if where_added { " AND type = ?" } else { " WHERE type = ?" }); }
                let mut q = sqlx::query_as::<_, (i64,)>(&sql);
                if let Some(level) = level { q = q.bind(level as i64); }
                if let Some(typ) = typ { q = q.bind(typ as i64); }
                let row = q.fetch_one(conn).await?;
                Ok::<(i64,), anyhow::Error>(row)
            })
        }).await?;
        Ok(cnt)
    }
}


