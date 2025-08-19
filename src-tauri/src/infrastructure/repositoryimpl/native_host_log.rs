use sqlx::query;
use async_trait::async_trait;

use super::repository::RepositoryImpl;
use crate::domain::repository::native_host_log::NativeHostLogRepository;
use crate::domain::native_host_log::{HostLogLevel, HostLogType};

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
}


