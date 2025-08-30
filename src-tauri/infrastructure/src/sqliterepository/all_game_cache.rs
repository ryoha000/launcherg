use chrono::{DateTime, Local};
use sqlx::Row;
use domain::{all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne}, repository::all_game_cache::AllGameCacheRepository};

use crate::sqliterepository::models::all_game_cache::AllGameCacheTable;
use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl AllGameCacheRepository for RepositoryImpl<domain::all_game_cache::AllGameCache> {
    async fn get_by_ids(&mut self, ids: Vec<i32>) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let rows = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut builder = sqlx::query_builder::QueryBuilder::new("SELECT id, gamename, thumbnail_url from all_game_caches WHERE id IN (");
                {
                    let mut separated = builder.separated(", ");
                    for id in ids.iter() {
                        separated.push_bind(id);
                    }
                }
                builder.push(")");
                let query = builder.build();
                let rows = query.fetch_all(conn).await?;
                Ok::<Vec<sqlx::sqlite::SqliteRow>, anyhow::Error>(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|row| AllGameCacheOneWithThumbnailUrl { id: row.get(0), gamename: row.get(1), thumbnail_url: row.get(2) }).collect())
    }

    async fn get_all(&mut self) -> anyhow::Result<AllGameCache> {
        let rows: Vec<AllGameCacheTable> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows = sqlx::query_as::<_, AllGameCacheTable>("select * from all_game_caches").fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().filter_map(|v| v.try_into().ok()).collect())
    }

    async fn get_last_updated(&mut self) -> anyhow::Result<(i32, DateTime<Local>)> {
        let (id, ts): (i32, sqlx::types::chrono::NaiveDateTime) = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let last_updated: (i32, sqlx::types::chrono::NaiveDateTime) = sqlx::query_as("SELECT MAX(id), MAX(created_at) from all_game_caches").fetch_one(conn).await?;
                Ok(last_updated)
            })
        }).await?;
        Ok((id, ts.and_utc().with_timezone(&Local)))
    }

    async fn update(&mut self, cache: Vec<NewAllGameCacheOne>) -> anyhow::Result<()> {
        if cache.is_empty() { return Ok(()); }
        for c in cache.chunks(1000) {
            let chunk = c.to_vec();
            self.executor.with_conn(|conn| {
                Box::pin(async move {
                    let mut qb = sqlx::query_builder::QueryBuilder::new("INSERT INTO all_game_caches (id, gamename, thumbnail_url) ");
                    qb.push_values(chunk, |mut b, new| { b.push_bind(new.id); b.push_bind(new.gamename.clone()); b.push_bind(new.thumbnail_url.clone()); });
                    qb.build().execute(conn).await?;
                    Ok::<(), anyhow::Error>(())
                })
            }).await?;
        }
        Ok(())
    }

    async fn delete_by_ids(&mut self, ids: Vec<i32>) -> anyhow::Result<()> {
        if ids.is_empty() { return Ok(()); }
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut qb = sqlx::query_builder::QueryBuilder::new("DELETE FROM all_game_caches WHERE id IN (");
                {
                    let mut separated = qb.separated(", ");
                    for id in ids.iter() { separated.push_bind(id); }
                }
                qb.push(")");
                qb.build().execute(conn).await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn search_by_name(&mut self, name: &str) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let name = name.to_string();
        let rows = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let search_pattern = format!("%{}%", name);
                let rows = sqlx::query(
                    "SELECT id, gamename, thumbnail_url FROM all_game_caches \\n+             WHERE gamename LIKE ? \\n+             ORDER BY \\n+                CASE \\n+                    WHEN gamename = ? THEN 1 \\n+                    WHEN gamename LIKE ? THEN 2 \\n+                    ELSE 3 \\n+                END, \\n+                LENGTH(gamename) \\n+             LIMIT 50"
                )
                .bind(&search_pattern)
                .bind(&name)
                .bind(format!("{}%", name))
                .fetch_all(conn)
                .await?;
                Ok::<Vec<sqlx::sqlite::SqliteRow>, anyhow::Error>(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|row| AllGameCacheOneWithThumbnailUrl { id: row.get(0), gamename: row.get(1), thumbnail_url: row.get(2) }).collect())
    }
}


