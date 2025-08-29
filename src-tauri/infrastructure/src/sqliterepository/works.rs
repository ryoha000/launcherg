use std::collections::BTreeMap;

use domain::{repositoryv2::works::WorkRepository, works::{DlsiteWork, DmmWork, NewWork, Work, WorkDetails}, Id};
use sqlx::{query_as, SqliteConnection};

use crate::{repositoryimpl::models::works::{WorkDetailsRow, WorkTable}, sqliterepository::sqliterepository::RepositoryImpl};

impl<'a> WorkRepository for RepositoryImpl<'a, domain::works::Work> {
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
                    title: entry.work.title.clone(),
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
                    title: entry.work.title.clone(),
                    store_id: r.dlsite_store_id.unwrap_or_default(),
                    category: r.dlsite_category.unwrap_or_default(),
                });
                entry.is_dlsite_omitted = r.dlsite_omit_id.is_some();
            }
        }

        Ok(map.into_values().collect())
    }
}
