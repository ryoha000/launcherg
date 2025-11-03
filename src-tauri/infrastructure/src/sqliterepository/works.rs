use std::collections::BTreeMap;

use domain::repository::work_lnk::{NewWorkLnk, WorkLnk as DomainWorkLnk, WorkLnkRepository};
use domain::work_link_pending_exe::WorkLinkPendingExeRepository;
use domain::{
    repository::works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
    works::{
        DlsiteWork, DmmWork, NewDlsiteWork, NewDmmWork, NewWork, NewWorkLike, Work, WorkDetails,
        WorkLike,
    },
    Id, StrId,
};
use sqlx::query_as;

use crate::sqliterepository::{
    models::works::{WorkDetailsRow, WorkLinkPendingExeRow, WorkLnkRow, WorkTable},
    sqliterepository::RepositoryImpl,
};

impl WorkRepository for RepositoryImpl<Work> {
    async fn upsert(&mut self, new_work: &NewWork) -> anyhow::Result<StrId<Work>> {
        let title = new_work.title.clone();
        let work_id = uuid::Uuid::new_v4().to_string();
        let id = self
            .executor
            .with_conn(|conn| {
                let work_id = work_id.clone();
                Box::pin(async move {
                    let (id,): (String,) = sqlx::query_as(
                        r#"INSERT INTO works (id, title) VALUES (?, ?)
                       RETURNING id"#,
                    )
                    .bind(work_id)
                    .bind(title)
                    .fetch_one(conn)
                    .await?;
                    Ok::<String, anyhow::Error>(id)
                })
            })
            .await?;

        Ok(StrId::new(id))
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
                        NULL as ce_created_at,
                        dw.id   as dmm_id,
                        dw.store_id as dmm_store_id,
                        dw.category as dmm_category,
                        dw.subcategory as dmm_subcategory,
                        NULL as ce_id,
                        wem.id as egs_id,
                        wem.erogamescape_id as egs_erogamescape_id,
                        wem.created_at as egs_created_at,
                        wem.updated_at as egs_updated_at,
                        ei.gamename_ruby as egs_info_gamename_ruby,
                        ei.brandname as egs_info_brandname,
                        ei.brandname_ruby as egs_info_brandname_ruby,
                        ei.sellday as egs_info_sellday,
                        ei.is_nukige as egs_info_is_nukige,
                        ei.created_at as egs_info_created_at,
                        ei.updated_at as egs_info_updated_at,
                        wt.thumbnail_width as cet_width,
                        wt.thumbnail_height as cet_height,
                        wi.install_at as install_install_at,
                        wp.last_play_at as play_last_play_at,
                        oo.id as omit_id,
                        pp.id as dmm_pack_id,
                        lw.id   as dlsite_id,
                        lw.store_id as dlsite_store_id,
                        lw.category as dlsite_category,
                        (SELECT id FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_id,
                        (SELECT download_path FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_download_path,
                        wl.id as like_id,
                        wl.like_at as like_like_at,
                        wl.created_at as like_created_at,
                        wl.updated_at as like_updated_at
                    FROM works w
                    LEFT JOIN dmm_works dw ON dw.work_id = w.id
                    LEFT JOIN work_erogamescape_map wem ON wem.work_id = w.id
                    LEFT JOIN erogamescape_information ei ON ei.id = wem.erogamescape_id
                    LEFT JOIN work_thumbnails AS wt ON wt.work_id = w.id
                    LEFT JOIN work_installs AS wi ON wi.work_id = w.id
                    LEFT JOIN work_plays AS wp ON wp.work_id = w.id
                    LEFT JOIN work_omits oo ON oo.work_id = w.id
                    LEFT JOIN dmm_work_packs pp ON pp.work_id = w.id
                    LEFT JOIN dlsite_works lw ON lw.work_id = w.id
                    LEFT JOIN work_likes wl ON wl.work_id = w.id
                    ORDER BY w.id ASC
                    "#,
                )
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        let mut map: BTreeMap<String, WorkDetails> = BTreeMap::new();
        for r in rows.into_iter() {
            let details: WorkDetails = r.into();
            let key = details.work.id.value.clone();
            match map.get_mut(&key) {
                Some(entry) => {
                    if entry.dmm.is_none() {
                        entry.dmm = details.dmm;
                    }
                    if entry.dlsite.is_none() {
                        entry.dlsite = details.dlsite;
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

    async fn find_details_by_work_id(
        &mut self,
        work_id: StrId<Work>,
    ) -> anyhow::Result<Option<WorkDetails>> {
        let idv = work_id.value.clone();
        let rows = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<WorkDetailsRow> = query_as(
                    r#"
                    SELECT 
                        w.id   as work_id,
                        w.title as work_title,
                        NULL as ce_created_at,
                        dw.id   as dmm_id,
                        dw.store_id as dmm_store_id,
                        dw.category as dmm_category,
                        dw.subcategory as dmm_subcategory,
                        NULL as ce_id,
                        wem.id as egs_id,
                        wem.erogamescape_id as egs_erogamescape_id,
                        wem.created_at as egs_created_at,
                        wem.updated_at as egs_updated_at,
                        ei.gamename_ruby as egs_info_gamename_ruby,
                        ei.brandname as egs_info_brandname,
                        ei.brandname_ruby as egs_info_brandname_ruby,
                        ei.sellday as egs_info_sellday,
                        ei.is_nukige as egs_info_is_nukige,
                        ei.created_at as egs_info_created_at,
                        ei.updated_at as egs_info_updated_at,
                        wt.thumbnail_width as cet_width,
                        wt.thumbnail_height as cet_height,
                        wi.install_at as install_install_at,
                        wp.last_play_at as play_last_play_at,
                        oo.id as omit_id,
                        pp.id as dmm_pack_id,
                        lw.id   as dlsite_id,
                        lw.store_id as dlsite_store_id,
                        lw.category as dlsite_category,
                        (SELECT id FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_id,
                        (SELECT download_path FROM work_download_paths wdp WHERE wdp.work_id = w.id ORDER BY id DESC LIMIT 1) as latest_path_download_path,
                        wl.id as like_id,
                        wl.like_at as like_like_at,
                        wl.created_at as like_created_at,
                        wl.updated_at as like_updated_at
                    FROM works w
                    LEFT JOIN dmm_works dw ON dw.work_id = w.id
                    LEFT JOIN work_erogamescape_map wem ON wem.work_id = w.id
                    LEFT JOIN erogamescape_information ei ON ei.id = wem.erogamescape_id
                    LEFT JOIN work_thumbnails AS wt ON wt.work_id = w.id
                    LEFT JOIN work_omits oo ON oo.work_id = w.id
                    LEFT JOIN dmm_work_packs pp ON pp.work_id = w.id
                    LEFT JOIN dlsite_works lw ON lw.work_id = w.id
                    LEFT JOIN work_installs AS wi ON wi.work_id = w.id
                    LEFT JOIN work_plays AS wp ON wp.work_id = w.id
                    LEFT JOIN work_likes wl ON wl.work_id = w.id
                    WHERE w.id = ?
                    ORDER BY w.id ASC
                    "#,
                )
                .bind(idv)
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        let mut map: BTreeMap<String, WorkDetails> = BTreeMap::new();
        for r in rows.into_iter() {
            let details: WorkDetails = r.into();
            let key = details.work.id.value.clone();
            match map.get_mut(&key) {
                Some(entry) => {
                    if entry.dmm.is_none() {
                        entry.dmm = details.dmm;
                    }
                    if entry.dlsite.is_none() {
                        entry.dlsite = details.dlsite;
                    }
                    if entry.erogamescape_information.is_none() {
                        entry.erogamescape_information = details.erogamescape_information;
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
    // 廃止: CE ID での検索は非対応

    async fn find_work_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> anyhow::Result<Vec<(i32, StrId<Work>)>> {
        use sqlx::QueryBuilder;
        if erogamescape_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = erogamescape_ids.to_vec();
        let rows: Vec<(i64, String)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let mut qb = QueryBuilder::new(
                        r#"SELECT wem.erogamescape_id, wem.work_id FROM work_erogamescape_map wem WHERE wem.erogamescape_id IN ("#,
                    );
                    {
                        let mut separated = qb.separated(", ");
                        for id in ids.iter() {
                            separated.push_bind(*id);
                        }
                    }
                    qb.push(")");
                    let rows: Vec<(i64, String)> = qb.build_query_as().fetch_all(conn).await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|(egs, wid)| (egs as i32, StrId::new(wid)))
            .collect())
    }

    async fn upsert_erogamescape_map(
        &mut self,
        work_id: StrId<Work>,
        erogamescape_id: i32,
    ) -> anyhow::Result<()> {
        let wid = work_id.value.clone();
        let egs_id = erogamescape_id as i64;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"INSERT INTO work_erogamescape_map (work_id, erogamescape_id)
                        VALUES (?, ?)
                        ON CONFLICT(work_id) DO UPDATE SET
                            erogamescape_id = excluded.erogamescape_id,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(wid)
                    .bind(egs_id)
                    .execute(&mut *conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn delete(&mut self, id: StrId<Work>) -> anyhow::Result<()> {
        let idv = id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"DELETE FROM works WHERE id = ?"#)
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
    }

    async fn list_work_ids_missing_thumbnail_size(&mut self) -> anyhow::Result<Vec<StrId<Work>>> {
        let rows: Vec<(String,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let rows: Vec<(String,)> = sqlx::query_as(
                        r#"
                        SELECT w.id
                        FROM works w
                        LEFT JOIN work_thumbnails wt ON wt.work_id = w.id
                        WHERE wt.work_id IS NULL 
                           OR wt.thumbnail_width IS NULL 
                           OR wt.thumbnail_height IS NULL
                        "#,
                    )
                    .fetch_all(conn)
                    .await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows.into_iter().map(|(id,)| StrId::new(id)).collect())
    }

    async fn upsert_work_thumbnail_size(
        &mut self,
        work_id: StrId<Work>,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        let wid = work_id.value.clone();
        let width = width as i64;
        let height = height as i64;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
                        INSERT INTO work_thumbnails (work_id, thumbnail_width, thumbnail_height)
                        VALUES (?, ?, ?)
                        ON CONFLICT(work_id) DO UPDATE SET
                            thumbnail_width = excluded.thumbnail_width,
                            thumbnail_height = excluded.thumbnail_height,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(wid)
                    .bind(width)
                    .bind(height)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn update_last_play_at_by_work_id(
        &mut self,
        work_id: StrId<Work>,
        last_play_at: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        let wid = work_id.value.clone();
        let last_play_at_naive = last_play_at.naive_utc();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
                        INSERT INTO work_plays (work_id, last_play_at)
                        VALUES (?, ?)
                        ON CONFLICT(work_id) DO UPDATE SET
                            last_play_at = excluded.last_play_at,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(wid)
                    .bind(last_play_at_naive)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn update_install_by_work_id(
        &mut self,
        work_id: StrId<Work>,
        install_at: chrono::DateTime<chrono::Local>,
        original_path: String,
    ) -> anyhow::Result<()> {
        let wid = work_id.value.clone();
        let install_at_naive = install_at.naive_utc();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
                        INSERT INTO work_installs (work_id, install_at, original_path)
                        VALUES (?, ?, ?)
                        ON CONFLICT(work_id) DO UPDATE SET
                            install_at = excluded.install_at,
                            original_path = excluded.original_path,
                            updated_at = CURRENT_TIMESTAMP
                        "#,
                    )
                    .bind(wid)
                    .bind(install_at_naive)
                    .bind(original_path)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
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
                    .bind(new_work.work_id.value.clone())
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

    async fn find_by_store_id(&mut self, store_id: &str) -> anyhow::Result<Option<DmmWork>> {
        let store_id = store_id.to_string();
        let row = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::DmmWorkTable> =
                        sqlx::query_as(
                            r#"SELECT w.id as id, w.store_id, w.category, w.subcategory, w.work_id
                       FROM dmm_works w
                       WHERE w.store_id=?
                       LIMIT 1"#,
                        )
                        .bind(store_id)
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
                        r#"WITH keys(store_id, category, subcategory) AS (VALUES "#,
                    );
                    for (idx, (store_id, category, subcategory)) in keys.iter().enumerate() {
                        if idx > 0 {
                            qb.push(", ");
                        }
                        qb.push("(");
                        qb.push_bind(store_id);
                        qb.push(", ");
                        qb.push_bind(category);
                        qb.push(", ");
                        qb.push_bind(subcategory);
                        qb.push(")");
                    }
                    qb.push(
                        r#")
SELECT w.id as id, w.store_id, w.category, w.subcategory, w.work_id
FROM dmm_works w
INNER JOIN keys k
  ON k.store_id = w.store_id
 AND k.category = w.category
 AND k.subcategory = w.subcategory"#,
                    );

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

    async fn find_by_work_id(&mut self, work_id: StrId<Work>) -> anyhow::Result<Option<DmmWork>> {
        let idv = work_id.value.clone();
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
                    .bind(new_work.work_id.value.clone())
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

    async fn find_by_store_id(&mut self, store_id: &str) -> anyhow::Result<Option<DlsiteWork>> {
        let store_id = store_id.to_string();
        let row = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::DlsiteWorkTable> =
                        sqlx::query_as(
                            r#"SELECT w.id as id, w.store_id, w.category, w.work_id
                       FROM dlsite_works w
                       WHERE w.store_id=?
                       LIMIT 1"#,
                        )
                        .bind(store_id)
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
        work_id: StrId<domain::works::Work>,
    ) -> anyhow::Result<Vec<DomainWorkLnk>> {
        let idv = work_id.value.clone();
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
        let work_id = new_lnk.work_id.value.clone();
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

impl WorkLinkPendingExeRepository
    for RepositoryImpl<domain::work_link_pending_exe::WorkLinkPendingExe>
{
    async fn list_all(
        &mut self,
    ) -> anyhow::Result<Vec<domain::work_link_pending_exe::WorkLinkPendingExe>> {
        let rows: Vec<WorkLinkPendingExeRow> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let rows: Vec<WorkLinkPendingExeRow> = sqlx::query_as(
                    r#"SELECT id, work_id, exe_path FROM work_link_pending_exe ORDER BY id ASC"#,
                )
                .fetch_all(conn)
                .await?;
                    Ok(rows)
                })
            })
            .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn delete(
        &mut self,
        id: domain::Id<domain::work_link_pending_exe::WorkLinkPendingExe>,
    ) -> anyhow::Result<()> {
        let idv = id.value as i64;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"DELETE FROM work_link_pending_exe WHERE id = ?"#)
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .await
    }
}

impl domain::repository::work_like::WorkLikeRepository for RepositoryImpl<domain::works::WorkLike> {
    async fn upsert(&mut self, like: &NewWorkLike) -> anyhow::Result<Id<WorkLike>> {
        let work_id = like.work_id.value.clone();
        let like_at = like.like_at.naive_utc();
        let (id,): (i64,) = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let (id,): (i64,) = sqlx::query_as(
                        r#"INSERT INTO work_likes (work_id, like_at) VALUES (?, ?)
                        ON CONFLICT(work_id) DO UPDATE SET like_at = excluded.like_at, updated_at = CURRENT_TIMESTAMP
                        RETURNING id"#,
                    )
                    .bind(work_id)
                    .bind(like_at)
                    .fetch_one(conn)
                    .await?;
                    Ok::<(i64,), anyhow::Error>((id,))
                })
            })
            .await?;
        Ok(Id::new(id as i32))
    }

    async fn delete_by_work_id(&mut self, work_id: StrId<Work>) -> anyhow::Result<()> {
        let idv = work_id.value.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(r#"DELETE FROM work_likes WHERE work_id = ?"#)
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
    }

    async fn get_by_work_id(&mut self, work_id: StrId<Work>) -> anyhow::Result<Option<WorkLike>> {
        let idv = work_id.value.clone();
        let row: Option<crate::sqliterepository::models::works::WorkLikeRow> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let row: Option<crate::sqliterepository::models::works::WorkLikeRow> =
                        sqlx::query_as(
                            r#"SELECT id, work_id, like_at, created_at, updated_at FROM work_likes WHERE work_id = ? LIMIT 1"#,
                        )
                        .bind(idv)
                        .fetch_optional(conn)
                        .await?;
                    Ok(row)
                })
            })
            .await?;
        Ok(row.map(|r| r.try_into()).transpose()?)
    }

    async fn update_like_at_by_work_id(
        &mut self,
        work_id: StrId<Work>,
        like_at: Option<chrono::DateTime<chrono::Local>>,
    ) -> anyhow::Result<()> {
        match like_at {
            Some(at) => {
                let like = NewWorkLike::new(work_id, at);
                let _ = self.upsert(&like).await?;
                Ok(())
            }
            None => self.delete_by_work_id(work_id).await,
        }
    }
}
