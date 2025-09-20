use std::collections::BTreeMap;

use domain::collection::CollectionElement;
use domain::repository::work_lnk::{NewWorkLnk, WorkLnk as DomainWorkLnk, WorkLnkRepository};
use domain::{
    repository::works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
    works::{DlsiteWork, DmmWork, NewDlsiteWork, NewDmmWork, NewWork, Work, WorkDetails},
    Id,
};
use sqlx::query_as;

use crate::sqliterepository::{
    models::works::{WorkDetailsRow, WorkLnkRow, WorkTable},
    sqliterepository::RepositoryImpl,
};

impl WorkRepository for RepositoryImpl<Work> {
    async fn upsert(&mut self, new_work: &NewWork) -> anyhow::Result<Id<Work>> {
        let title = new_work.title.clone();
        let id = self
            .executor
            .with_conn(|conn| {
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
            })
            .await?;

        Ok(Id::new(id as i32))
    }

    async fn find_by_title(&mut self, title: &str) -> anyhow::Result<Option<Work>> {
        let title = title.to_string();
        let row: Option<WorkTable> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<WorkTable> =
                        query_as(r#"SELECT id, title FROM works WHERE title=? LIMIT 1"#)
                            .bind(title)
                            .fetch_optional(conn)
                            .await?;
                    Ok(row)
                })
            })
            .await?;

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
                        m1.collection_element_id as ce_id,
                        e.id as egs_id,
                        e.erogamescape_id as egs_erogamescape_id,
                        e.created_at as egs_created_at,
                        e.updated_at as egs_updated_at,
                        oo.id as omit_id,
                        pp.id as dmm_pack_id,
                        lw.id   as dlsite_id,
                        lw.store_id as dlsite_store_id,
                        lw.category as dlsite_category,
                        (SELECT id FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_id,
                        (SELECT download_path FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_download_path
                    FROM works w
                    LEFT JOIN dmm_works dw ON dw.work_id = w.id
                    LEFT JOIN work_collection_elements m1 ON m1.work_id = w.id
                    LEFT JOIN collection_element_erogamescape_map e ON e.collection_element_id = m1.collection_element_id
                    LEFT JOIN work_omits oo ON oo.work_id = w.id
                    LEFT JOIN dmm_work_packs pp ON pp.work_id = w.id
                    LEFT JOIN dlsite_works lw ON lw.work_id = w.id
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
            let details: WorkDetails = r.into();
            let key = details.work.id.value as i64;
            match map.get_mut(&key) {
                Some(entry) => {
                    if entry.dmm.is_none() {
                        entry.dmm = details.dmm;
                    }
                    if entry.dlsite.is_none() {
                        entry.dlsite = details.dlsite;
                    }
                    if entry.collection_element_id.is_none() {
                        entry.collection_element_id = details.collection_element_id;
                    }
                    if entry.erogamescape.is_none() {
                        entry.erogamescape = details.erogamescape;
                    }
                    entry.is_omitted |= details.is_omitted;
                    entry.is_dmm_pack |= details.is_dmm_pack;
                    if entry.latest_download_path.is_none() {
                        entry.latest_download_path = details.latest_download_path;
                    }
                }
                None => {
                    map.insert(key, details);
                }
            }
        }

        Ok(map.into_values().collect())
    }

    async fn find_details_by_collection_element_id(
        &mut self,
        collection_element_id: Id<CollectionElement>,
    ) -> anyhow::Result<Option<WorkDetails>> {
        let idv = collection_element_id.value as i64;
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
                        m1.collection_element_id as ce_id,
                        e.id as egs_id,
                        e.erogamescape_id as egs_erogamescape_id,
                        e.created_at as egs_created_at,
                        e.updated_at as egs_updated_at,
                        oo.id as omit_id,
                        pp.id as dmm_pack_id,
                        lw.id   as dlsite_id,
                        lw.store_id as dlsite_store_id,
                        lw.category as dlsite_category,
                        (SELECT id FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_id,
                        (SELECT download_path FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_download_path
                    FROM works w
                    JOIN work_collection_elements m1 ON m1.work_id = w.id
                    LEFT JOIN dmm_works dw ON dw.work_id = w.id
                    LEFT JOIN collection_element_erogamescape_map e ON e.collection_element_id = m1.collection_element_id
                    LEFT JOIN work_omits oo ON oo.work_id = w.id
                    LEFT JOIN dmm_work_packs pp ON pp.work_id = w.id
                    LEFT JOIN dlsite_works lw ON lw.work_id = w.id
                    WHERE m1.collection_element_id = ?
                    LIMIT 1
                    "#,
                )
                .bind(idv)
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        let mut map: std::collections::BTreeMap<i64, WorkDetails> =
            std::collections::BTreeMap::new();
        for r in rows.into_iter() {
            let details: WorkDetails = r.into();
            let key = details.work.id.value as i64;
            match map.get_mut(&key) {
                Some(entry) => {
                    if entry.dmm.is_none() {
                        entry.dmm = details.dmm;
                    }
                    if entry.dlsite.is_none() {
                        entry.dlsite = details.dlsite;
                    }
                    if entry.collection_element_id.is_none() {
                        entry.collection_element_id = details.collection_element_id;
                    }
                    if entry.erogamescape.is_none() {
                        entry.erogamescape = details.erogamescape;
                    }
                    entry.is_omitted |= details.is_omitted;
                    entry.is_dmm_pack |= details.is_dmm_pack;
                    if entry.latest_download_path.is_none() {
                        entry.latest_download_path = details.latest_download_path;
                    }
                }
                None => {
                    map.insert(key, details);
                }
            }
        }
        Ok(map.into_values().next())
    }
}

impl DmmWorkRepository for RepositoryImpl<domain::works::DmmWork> {
    async fn upsert(&mut self, new_work: &NewDmmWork) -> anyhow::Result<Id<DmmWork>> {
        let new_work = new_work.clone();
        let dmm_id: i64 = self
            .executor
            .with_conn(|conn| {
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
            })
            .await?;

        Ok(Id::new(dmm_id as i32))
    }

    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
        subcategory: &str,
    ) -> anyhow::Result<Option<DmmWork>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let subcategory = subcategory.to_string();
        let row = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::DmmWorkTable> =
                        sqlx::query_as(
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
            })
            .await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }

    async fn find_by_store_keys(
        &mut self,
        keys: &[(String, String, String)],
    ) -> anyhow::Result<Vec<DmmWork>> {
        use sqlx::QueryBuilder;
        if keys.is_empty() {
            return Ok(Vec::new());
        }
        let keys = keys.to_vec();
        let rows = self
            .executor
            .with_conn(|conn| {
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

                    let rows: Vec<crate::sqliterepository::models::works::DmmWorkTable> =
                        qb.build_query_as().fetch_all(conn).await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|t| t.try_into())
            .collect::<anyhow::Result<Vec<_>>>()?)
    }

    async fn find_by_work_id(&mut self, work_id: Id<Work>) -> anyhow::Result<Option<DmmWork>> {
        let idv = work_id.value as i64;
        let row = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::DmmWorkTable> =
                        sqlx::query_as(
                            r#"SELECT w.id as id, w.store_id, w.category, w.subcategory, w.work_id
                       FROM dmm_works w
                       WHERE w.work_id=?
                       LIMIT 1"#,
                        )
                        .bind(idv)
                        .fetch_optional(conn)
                        .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }
}

impl DlsiteWorkRepository for RepositoryImpl<domain::works::DlsiteWork> {
    async fn upsert(&mut self, new_work: &NewDlsiteWork) -> anyhow::Result<Id<DlsiteWork>> {
        let new_work = new_work.clone();
        let dl_id: i64 = self
            .executor
            .with_conn(|conn| {
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
            })
            .await?;

        Ok(Id::new(dl_id as i32))
    }

    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
    ) -> anyhow::Result<Option<DlsiteWork>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let row = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::DlsiteWorkTable> =
                        sqlx::query_as(
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
            })
            .await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }
}

impl WorkLnkRepository for RepositoryImpl<domain::repository::work_lnk::WorkLnk> {
    async fn find_by_id(
        &mut self,
        id: Id<domain::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<Option<DomainWorkLnk>> {
        let idv = id.value as i64;
        let row: Option<WorkLnkRow> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<WorkLnkRow> = sqlx::query_as(
                        r#"SELECT id, work_id, lnk_path FROM work_lnks WHERE id = ? LIMIT 1"#,
                    )
                    .bind(idv)
                    .fetch_optional(conn)
                    .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|r| r.into()))
    }
    async fn list_by_work_id(
        &mut self,
        work_id: Id<domain::works::Work>,
    ) -> anyhow::Result<Vec<DomainWorkLnk>> {
        let idv = work_id.value as i64;
        let rows: Vec<WorkLnkRow> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<WorkLnkRow> = sqlx::query_as(
                    r#"SELECT id, work_id, lnk_path FROM work_lnks WHERE work_id = ? ORDER BY id ASC"#,
                )
                .bind(idv)
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn insert(
        &mut self,
        new_lnk: &NewWorkLnk,
    ) -> anyhow::Result<Id<domain::repository::work_lnk::WorkLnk>> {
        let work_id = new_lnk.work_id.value as i64;
        let lnk_path = new_lnk.lnk_path.clone();
        let id: i64 = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let (id,): (i64,) = sqlx::query_as(
                        r#"INSERT INTO work_lnks (work_id, lnk_path) VALUES (?, ?) RETURNING id"#,
                    )
                    .bind(work_id)
                    .bind(lnk_path)
                    .fetch_one(conn)
                    .await?;
                    Ok::<i64, anyhow::Error>(id)
                })
            })
            .await?;

        Ok(Id::new(id as i32))
    }

    async fn delete(
        &mut self,
        id: Id<domain::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<()> {
        let idv = id.value as i64;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"DELETE FROM work_lnks WHERE id = ?"#)
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .await
    }
}
