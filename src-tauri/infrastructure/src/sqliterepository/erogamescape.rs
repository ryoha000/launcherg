use domain::erogamescape::NewErogamescapeInformation;
use domain::repository::erogamescape::ErogamescapeRepository;

use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl ErogamescapeRepository for RepositoryImpl<domain::erogamescape::ErogamescapeInformation> {
    async fn upsert_information(
        &mut self,
        info: &NewErogamescapeInformation,
    ) -> anyhow::Result<()> {
        let req = info.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let is_nukige = if req.is_nukige { 1 } else { 0 };
                    sqlx::query(
                        r#"INSERT INTO erogamescape_information (id, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby)
                           VALUES (?, ?, ?, ?, ?, ?)
                           ON CONFLICT(id) DO UPDATE SET
                             gamename_ruby = excluded.gamename_ruby,
                             sellday = excluded.sellday,
                             is_nukige = excluded.is_nukige,
                             brandname = excluded.brandname,
                             brandname_ruby = excluded.brandname_ruby,
                             updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(req.erogamescape_id)
                    .bind(req.gamename_ruby)
                    .bind(req.sellday)
                    .bind(is_nukige)
                    .bind(req.brandname)
                    .bind(req.brandname_ruby)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn find_missing_information_ids(&mut self) -> anyhow::Result<Vec<i32>> {
        let rows: Vec<(i64,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let rows: Vec<(i64,)> = sqlx::query_as(
                        r#"SELECT wem.erogamescape_id
                            FROM work_erogamescape_map wem
                            LEFT JOIN erogamescape_information ei ON ei.id = wem.erogamescape_id
                            WHERE ei.id IS NULL"#,
                    )
                    .fetch_all(conn)
                    .await?;
                    Ok::<Vec<(i64,)>, anyhow::Error>(rows)
                })
            })
            .await?;
        Ok(rows.into_iter().map(|(v,)| v as i32).collect())
    }
}
