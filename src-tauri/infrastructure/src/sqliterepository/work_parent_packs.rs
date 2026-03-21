use crate::sqliterepository::sqliterepository::RepositoryImpl;
use domain::{
    repository::work_parent_packs::WorkParentPacksRepository,
    work_parent_pack::ParentPackKey,
    works::Work,
    StrId,
};

impl WorkParentPacksRepository for RepositoryImpl<domain::work_parent_pack::WorkParentPack> {
    async fn add(
        &mut self,
        work_id: StrId<Work>,
        parent_pack: ParentPackKey,
    ) -> anyhow::Result<()> {
        let work_id_str = work_id.value.clone();
        let ParentPackKey {
            store_id,
            category,
            subcategory,
        } = parent_pack;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"INSERT INTO work_parent_packs (work_id, parent_pack_store_id, parent_pack_category, parent_pack_subcategory)
                           VALUES (?, ?, ?, ?)
                           ON CONFLICT(work_id) DO UPDATE SET
                               parent_pack_store_id = excluded.parent_pack_store_id,
                               parent_pack_category = excluded.parent_pack_category,
                               parent_pack_subcategory = excluded.parent_pack_subcategory,
                               updated_at = CURRENT_TIMESTAMP"#,
                    )
                    .bind(work_id_str)
                    .bind(store_id)
                    .bind(category)
                    .bind(subcategory)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn exists(
        &mut self,
        work_id: StrId<Work>,
        parent_pack: ParentPackKey,
    ) -> anyhow::Result<bool> {
        let work_id_str = work_id.value.clone();
        let ParentPackKey {
            store_id,
            category,
            subcategory,
        } = parent_pack;
        let row: Option<(i64,)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(
                    r#"SELECT 1 FROM work_parent_packs
                       WHERE work_id=?
                         AND parent_pack_store_id=?
                         AND parent_pack_category=?
                         AND parent_pack_subcategory=?
                       LIMIT 1"#,
                )
                .bind(work_id_str)
                .bind(store_id)
                .bind(category)
                .bind(subcategory)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;
        Ok(row.is_some())
    }

    async fn find_parent_key(
        &mut self,
        work_id: StrId<Work>,
    ) -> anyhow::Result<Option<ParentPackKey>> {
        let wid = work_id.value.clone();
        let row: Option<(String, String, String)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<(String, String, String)> = sqlx::query_as(
                        r#"SELECT parent_pack_store_id, parent_pack_category, parent_pack_subcategory
                           FROM work_parent_packs
                           WHERE work_id=?
                           LIMIT 1"#,
                    )
                    .bind(wid)
                    .fetch_optional(conn)
                    .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|(store_id, category, subcategory)| ParentPackKey {
            store_id,
            category,
            subcategory,
        }))
    }
}
