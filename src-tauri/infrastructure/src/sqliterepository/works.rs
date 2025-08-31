use std::collections::BTreeMap;

use domain::{repository::works::{WorkRepository, DmmWorkRepository, DlsiteWorkRepository}, works::{DlsiteWork, DmmWork, NewDlsiteWork, NewDmmWork, NewWork, Work, WorkDetails}, Id};
use sqlx::query_as;

use crate::sqliterepository::{models::works::{WorkDetailsRow, WorkTable}, sqliterepository::RepositoryImpl};

impl WorkRepository for RepositoryImpl<Work> {
    async fn upsert(&mut self, new_work: &NewWork) -> anyhow::Result<Id<Work>> {
        let title = new_work.title.clone();
        let id = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let (id,): (i64,) = sqlx::query_as(
                    r#"INSERT INTO works (title) VALUES (?)
                       RETURNING id"#,
                )
                .bind(title)
                .fetch_one(conn)
                .await?;
                Ok::<i64, anyhow::Error>(id)
            })
        }).await?;

        Ok(Id::new(id as i32))
    }

    async fn find_by_title(&mut self, title: &str) -> anyhow::Result<Option<Work>> {
        let title = title.to_string();
        let row: Option<WorkTable> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let row: Option<WorkTable> = query_as(
                    r#"SELECT id, title FROM works WHERE title=? LIMIT 1"#,
                )
                .bind(title)
                .fetch_optional(conn)
                .await?;
                Ok(row)
            })
        }).await?;

        Ok(row.map(|t| t.try_into()).transpose()?)
    }

    async fn list_all_details(&mut self) -> anyhow::Result<Vec<WorkDetails>> {
        let rows = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<WorkDetailsRow> = query_as(
                    r#"
                    SELECT 
                        w.id   as work_id,
                        w.title as work_title,
                        dw.id   as dmm_id,
                        dw.store_id as dmm_store_id,
                        dw.category as dmm_category,
                        dw.subcategory as dmm_subcategory,
                        COALESCE(m1.collection_element_id, m2.collection_element_id) as ce_id,
                        oo.id as dmm_omit_id,
                        pp.id as dmm_pack_id,
                        lw.id   as dlsite_id,
                        lw.store_id as dlsite_store_id,
                        lw.category as dlsite_category,
                        oo2.id as dlsite_omit_id
                    FROM works w
                    LEFT JOIN dmm_works dw ON dw.work_id = w.id
                    LEFT JOIN work_collection_elements m1 ON m1.work_id = w.id
                    LEFT JOIN work_omits oo ON oo.work_id = w.id
                    LEFT JOIN dmm_work_packs pp ON pp.work_id = w.id
                    LEFT JOIN dlsite_works lw ON lw.work_id = w.id
                    LEFT JOIN work_collection_elements m2 ON m2.work_id = w.id
                    LEFT JOIN work_omits oo2 ON oo2.work_id = w.id
                    ORDER BY w.id ASC
                    "#,
                )
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        let mut map: BTreeMap<i64, WorkDetails> = BTreeMap::new();
        for r in rows.into_iter() {
            let entry = map.entry(r.work_id).or_insert_with(|| WorkDetails {
                work: Work { id: Id::new(r.work_id as i32), title: r.work_title.clone() },
                dmm: None,
                dlsite: None,
                collection_element_id: r.ce_id.map(|v| Id::new(v as i32)),
                is_dmm_omitted: false,
                is_dlsite_omitted: false,
                is_dmm_pack: false,
            });

            if let Some(dmm_id) = r.dmm_id {
                entry.dmm = Some(DmmWork {
                    id: Id::new(dmm_id as i32),
                    work_id: Id::new(r.work_id as i32),
                    store_id: r.dmm_store_id.unwrap_or_default(),
                    category: r.dmm_category.unwrap_or_default(),
                    subcategory: r.dmm_subcategory.unwrap_or_default(),
                });
                entry.is_dmm_omitted = r.dmm_omit_id.is_some();
                entry.is_dmm_pack = r.dmm_pack_id.is_some();
            }
            if let Some(dl_id) = r.dlsite_id {
                entry.dlsite = Some(DlsiteWork {
                    id: Id::new(dl_id as i32),
                    work_id: Id::new(r.work_id as i32),
                    store_id: r.dlsite_store_id.unwrap_or_default(),
                    category: r.dlsite_category.unwrap_or_default(),
                });
                entry.is_dlsite_omitted = r.dlsite_omit_id.is_some();
            }
        }

        Ok(map.into_values().collect())
    }
}

impl DmmWorkRepository for RepositoryImpl<domain::works::DmmWork> {
    async fn upsert(&mut self, new_work: &NewDmmWork) -> anyhow::Result<Id<DmmWork>> {
        let new_work = new_work.clone();
        let dmm_id: i64 = self.executor.with_conn(|conn| {
            Box::pin(async move {
                // dmm_works を UPSERT。RETURNING で常に行を返す
                let (id,): (i64,) = query_as(
                    r#"INSERT INTO dmm_works (store_id, category, subcategory, work_id)
                        VALUES (?, ?, ?, ?)
                        ON CONFLICT(store_id) DO UPDATE SET
                            category = excluded.category,
                            subcategory = excluded.subcategory,
                            work_id = excluded.work_id,
                            updated_at = CURRENT_TIMESTAMP
                        RETURNING id"#,
                )
                .bind(&new_work.store_id)
                .bind(&new_work.category)
                .bind(&new_work.subcategory)
                .bind(new_work.work_id.value as i64)
                .fetch_one(&mut *conn)
                .await?;

                Ok::<i64, anyhow::Error>(id)
            })
        }).await?;

        Ok(Id::new(dmm_id as i32))
    }

    async fn find_by_store_key(&mut self, store_id: &str, category: &str, subcategory: &str) -> anyhow::Result<Option<DmmWork>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let subcategory = subcategory.to_string();
        let row = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let row: Option<crate::sqliterepository::models::works::DmmWorkTable> = sqlx::query_as(
                    r#"SELECT w.id as id, w.store_id, w.category, w.subcategory, w.work_id
                       FROM dmm_works w
                       WHERE w.store_id=? AND w.category=? AND w.subcategory=?
                       LIMIT 1"#,
                )
                .bind(store_id)
                .bind(category)
                .bind(subcategory)
                .fetch_optional(conn)
                .await?;
                Ok(row)
            })
        }).await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }

    async fn find_by_store_keys(&mut self, keys: &[(String, String, String)]) -> anyhow::Result<Vec<DmmWork>> {
        use sqlx::QueryBuilder;
        if keys.is_empty() { return Ok(Vec::new()); }
        let keys = keys.to_vec();
        let rows = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut qb = QueryBuilder::new(
                    r#"SELECT w.id as id, w.store_id, w.category, w.subcategory, w.work_id
                        FROM dmm_works w
                        WHERE (w.store_id, w.category, w.subcategory) IN ("#,
                );
                {
                    let mut separated = qb.separated(", ");
                    for (store_id, category, subcategory) in keys.iter() {
                        separated.push_unseparated("(");
                        separated.push_bind(store_id);
                        separated.push_unseparated(", ");
                        separated.push_bind(category);
                        separated.push_unseparated(", ");
                        separated.push_bind(subcategory);
                        separated.push_unseparated(")");
                    }
                }
                qb.push(")");

                let rows: Vec<crate::sqliterepository::models::works::DmmWorkTable> = qb
                    .build_query_as()
                    .fetch_all(conn)
                    .await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|t| t.try_into()).collect::<anyhow::Result<Vec<_>>>()?)
    }
}

impl DlsiteWorkRepository for RepositoryImpl<domain::works::DlsiteWork> {
    async fn upsert(&mut self, new_work: &NewDlsiteWork) -> anyhow::Result<Id<DlsiteWork>> {
        let new_work = new_work.clone();
        let dl_id: i64 = self.executor.with_conn(|conn| {
            Box::pin(async move {
                // dlsite_works を UPSERT。RETURNING で常に行を返す
                let (id,): (i64,) = query_as(
                    r#"INSERT INTO dlsite_works (store_id, category, work_id)
                        VALUES (?, ?, ?)
                        ON CONFLICT(store_id) DO UPDATE SET
                            category = excluded.category,
                            work_id = excluded.work_id,
                            updated_at = CURRENT_TIMESTAMP
                        RETURNING id"#,
                )
                .bind(&new_work.store_id)
                .bind(&new_work.category)
                .bind(new_work.work_id.value as i64)
                .fetch_one(&mut *conn)
                .await?;

                Ok::<i64, anyhow::Error>(id)
            })
        }).await?;

        Ok(Id::new(dl_id as i32))
    }

    async fn find_by_store_key(&mut self, store_id: &str, category: &str) -> anyhow::Result<Option<DlsiteWork>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let row = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let row: Option<crate::sqliterepository::models::works::DlsiteWorkTable> = sqlx::query_as(
                    r#"SELECT w.id as id, w.store_id, w.category, w.work_id
                       FROM dlsite_works w
                       WHERE w.store_id=? AND w.category=?
                       LIMIT 1"#,
                )
                .bind(store_id)
                .bind(category)
                .fetch_optional(conn)
                .await?;
                Ok(row)
            })
        }).await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }
}
