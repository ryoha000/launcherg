use chrono::{DateTime, Local};
use domain::repository::collection::CollectionRepository;
use domain::{
    collection::{
        CollectionElement, CollectionElementErogamescape, CollectionElementInfo,
        CollectionElementInstall, CollectionElementLike, CollectionElementPaths,
        CollectionElementPlay, CollectionElementThumbnail, NewCollectionElement,
        NewCollectionElementInfo, NewCollectionElementInstall, NewCollectionElementLike,
        NewCollectionElementPaths, NewCollectionElementPlay, NewCollectionElementThumbnail,
    },
    works::Work,
    Id,
};
use sqlx::{query, query_as};

use crate::sqliterepository::models::collection::{
    CollectionElementDetailsRow, CollectionElementErogamescapeTable, CollectionElementInfoTable,
    CollectionElementInstallTable, CollectionElementLikeTable, CollectionElementPathsTable,
    CollectionElementPlayTable, CollectionElementThumbnailTable,
};
use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl CollectionRepository for RepositoryImpl<domain::collection::CollectionElement> {
    async fn get_all_elements(&mut self) -> anyhow::Result<Vec<CollectionElement>> {
        use std::collections::BTreeMap;
        let rows: Vec<CollectionElementDetailsRow> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<CollectionElementDetailsRow> = query_as(
                    r#"
                    SELECT 
                        ce.id AS ce_id,
                        ce.gamename AS ce_gamename,
                        ce.created_at AS ce_created_at,
                        ce.updated_at AS ce_updated_at,

                        cei.id AS info_id,
                        cei.gamename_ruby AS info_gamename_ruby,
                        cei.sellday AS info_sellday,
                        cei.is_nukige AS info_is_nukige,
                        cei.brandname AS info_brandname,
                        cei.brandname_ruby AS info_brandname_ruby,
                        cei.created_at AS info_created_at,
                        cei.updated_at AS info_updated_at,

                        cep.id AS paths_id,
                        cep.exe_path AS paths_exe_path,
                        cep.lnk_path AS paths_lnk_path,
                        cep.created_at AS paths_created_at,
                        cep.updated_at AS paths_updated_at,

                        cei_install.id AS install_id,
                        cei_install.install_at AS install_install_at,
                        cei_install.created_at AS install_created_at,
                        cei_install.updated_at AS install_updated_at,

                        cei_play.id AS play_id,
                        cei_play.last_play_at AS play_last_play_at,
                        cei_play.created_at AS play_created_at,
                        cei_play.updated_at AS play_updated_at,

                        cei_like.id AS like_id,
                        cei_like.like_at AS like_like_at,
                        cei_like.created_at AS like_created_at,
                        cei_like.updated_at AS like_updated_at,

                        cet.id AS thumbnail_id,
                        cet.thumbnail_width AS thumbnail_width,
                        cet.thumbnail_height AS thumbnail_height,
                        cet.created_at AS thumbnail_created_at,
                        cet.updated_at AS thumbnail_updated_at,

                        cee.id AS egs_id,
                        cee.erogamescape_id AS egs_erogamescape_id,
                        cee.created_at AS egs_created_at,
                        cee.updated_at AS egs_updated_at
                    FROM collection_elements ce
                    LEFT JOIN collection_element_info_by_erogamescape cei ON ce.id = cei.collection_element_id
                    LEFT JOIN collection_element_paths cep ON ce.id = cep.collection_element_id
                    LEFT JOIN collection_element_installs cei_install ON ce.id = cei_install.collection_element_id
                    LEFT JOIN collection_element_plays cei_play ON ce.id = cei_play.collection_element_id
                    LEFT JOIN collection_element_likes cei_like ON ce.id = cei_like.collection_element_id
                    LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
                    LEFT JOIN collection_element_erogamescape_map cee ON ce.id = cee.collection_element_id
                    ORDER BY ce.id ASC
                    "#,
                )
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        let mut map: BTreeMap<i32, CollectionElement> = BTreeMap::new();
        for r in rows.into_iter() {
            let element: CollectionElement = r.into();
            let key = element.id.value;
            match map.get_mut(&key) {
                Some(entry) => {
                    if entry.info.is_none() {
                        entry.info = element.info;
                    }
                    if entry.paths.is_none() {
                        entry.paths = element.paths;
                    }
                    if entry.install.is_none() {
                        entry.install = element.install;
                    }
                    if entry.play.is_none() {
                        entry.play = element.play;
                    }
                    if entry.like.is_none() {
                        entry.like = element.like;
                    }
                    if entry.thumbnail.is_none() {
                        entry.thumbnail = element.thumbnail;
                    }
                    if entry.erogamescape.is_none() {
                        entry.erogamescape = element.erogamescape;
                    }
                }
                None => {
                    map.insert(key, element);
                }
            }
        }
        Ok(map.into_values().collect())
    }

    async fn get_element_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElement>> {
        let idv = id.value;
        let rows: Vec<CollectionElementDetailsRow> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let rows: Vec<CollectionElementDetailsRow> = query_as(
                    r#"
                    SELECT 
                        ce.id AS ce_id,
                        ce.gamename AS ce_gamename,
                        ce.created_at AS ce_created_at,
                        ce.updated_at AS ce_updated_at,

                        cei.id AS info_id,
                        cei.gamename_ruby AS info_gamename_ruby,
                        cei.sellday AS info_sellday,
                        cei.is_nukige AS info_is_nukige,
                        cei.brandname AS info_brandname,
                        cei.brandname_ruby AS info_brandname_ruby,
                        cei.created_at AS info_created_at,
                        cei.updated_at AS info_updated_at,

                        cep.id AS paths_id,
                        cep.exe_path AS paths_exe_path,
                        cep.lnk_path AS paths_lnk_path,
                        cep.created_at AS paths_created_at,
                        cep.updated_at AS paths_updated_at,

                        cei_install.id AS install_id,
                        cei_install.install_at AS install_install_at,
                        cei_install.created_at AS install_created_at,
                        cei_install.updated_at AS install_updated_at,

                        cei_play.id AS play_id,
                        cei_play.last_play_at AS play_last_play_at,
                        cei_play.created_at AS play_created_at,
                        cei_play.updated_at AS play_updated_at,

                        cei_like.id AS like_id,
                        cei_like.like_at AS like_like_at,
                        cei_like.created_at AS like_created_at,
                        cei_like.updated_at AS like_updated_at,

                        cet.id AS thumbnail_id,
                        cet.thumbnail_width AS thumbnail_width,
                        cet.thumbnail_height AS thumbnail_height,
                        cet.created_at AS thumbnail_created_at,
                        cet.updated_at AS thumbnail_updated_at,

                        cee.id AS egs_id,
                        cee.erogamescape_id AS egs_erogamescape_id,
                        cee.created_at AS egs_created_at,
                        cee.updated_at AS egs_updated_at
                    FROM collection_elements ce
                    LEFT JOIN collection_element_info_by_erogamescape cei ON ce.id = cei.collection_element_id
                    LEFT JOIN collection_element_paths cep ON ce.id = cep.collection_element_id
                    LEFT JOIN collection_element_installs cei_install ON ce.id = cei_install.collection_element_id
                    LEFT JOIN collection_element_plays cei_play ON ce.id = cei_play.collection_element_id
                    LEFT JOIN collection_element_likes cei_like ON ce.id = cei_like.collection_element_id
                    LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
                    LEFT JOIN collection_element_erogamescape_map cee ON ce.id = cee.collection_element_id
                    WHERE ce.id = ?
                    "#,
                )
                .bind(idv)
                .fetch_all(conn)
                .await?;
                Ok(rows)
            })
        }).await?;

        if rows.is_empty() {
            return Ok(None);
        }
        // 同一IDで重複が出た場合は最初の要素に統合
        let mut iter = rows.into_iter();
        let mut element: CollectionElement = iter.next().unwrap().into();
        for r in iter {
            let e: CollectionElement = r.into();
            if element.info.is_none() {
                element.info = e.info;
            }
            if element.paths.is_none() {
                element.paths = e.paths;
            }
            if element.install.is_none() {
                element.install = e.install;
            }
            if element.play.is_none() {
                element.play = e.play;
            }
            if element.like.is_none() {
                element.like = e.like;
            }
            if element.thumbnail.is_none() {
                element.thumbnail = e.thumbnail;
            }
            if element.erogamescape.is_none() {
                element.erogamescape = e.erogamescape;
            }
        }
        Ok(Some(element))
    }

    async fn upsert_collection_element(
        &mut self,
        new_element: &NewCollectionElement,
    ) -> anyhow::Result<()> {
        let new = new_element.clone();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query(
                    "INSERT INTO collection_elements (id, gamename) VALUES (?, ?) 
             ON CONFLICT(id) DO UPDATE SET gamename = excluded.gamename, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(new.id.value)
                .bind(&new.gamename)
                .execute(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn update_collection_element_gamename_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        gamename: &str,
    ) -> anyhow::Result<()> {
        let idv = id.value;
        let name = gamename.to_string();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query("UPDATE collection_elements SET gamename = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(name)
                    .bind(idv)
                    .execute(conn)
                    .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn delete_collection_element(
        &mut self,
        element_id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let idv = element_id.value;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query("DELETE FROM collection_elements WHERE id = ?")
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn upsert_collection_element_info(
        &mut self,
        info: &NewCollectionElementInfo,
    ) -> anyhow::Result<()> {
        let i = info.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query(
                        "INSERT INTO collection_element_info_by_erogamescape 
             (collection_element_id, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby) 
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             gamename_ruby = ?, sellday = ?, is_nukige = ?, 
             brandname = ?, brandname_ruby = ?, updated_at = CURRENT_TIMESTAMP",
                    )
                    .bind(i.collection_element_id.value)
                    .bind(&i.gamename_ruby)
                    .bind(&i.sellday)
                    .bind(if i.is_nukige { 1 } else { 0 })
                    .bind(&i.brandname)
                    .bind(&i.brandname_ruby)
                    .bind(&i.gamename_ruby)
                    .bind(&i.sellday)
                    .bind(if i.is_nukige { 1 } else { 0 })
                    .bind(&i.brandname)
                    .bind(&i.brandname_ruby)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_element_info_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementInfo>> {
        let idv = id.value;
        let info_table: Option<CollectionElementInfoTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_info_by_erogamescape WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(info_table.map(|t| t.try_into()).transpose()?)
    }

    async fn get_not_registered_info_element_ids(
        &mut self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let ids: Vec<(i32,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(sqlx::query_as(
                        "SELECT ce.id
            FROM collection_elements ce
            LEFT JOIN collection_element_info_by_erogamescape cei
            ON ce.id = cei.collection_element_id
            WHERE cei.collection_element_id IS NULL",
                    )
                    .fetch_all(conn)
                    .await?)
                })
            })
            .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn upsert_collection_element_paths(
        &mut self,
        paths: &NewCollectionElementPaths,
    ) -> anyhow::Result<()> {
        let p = paths.clone();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query(
                    "INSERT INTO collection_element_paths (collection_element_id, exe_path, lnk_path) 
             VALUES (?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             exe_path = ?, lnk_path = ?, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(p.collection_element_id.value)
                .bind(&p.exe_path)
                .bind(&p.lnk_path)
                .bind(&p.exe_path)
                .bind(&p.lnk_path)
                .execute(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn get_element_paths_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementPaths>> {
        let idv = id.value;
        let paths_table: Option<CollectionElementPathsTable> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(query_as(
                        "SELECT * FROM collection_element_paths WHERE collection_element_id = ?",
                    )
                    .bind(idv)
                    .fetch_optional(conn)
                    .await?)
                })
            })
            .await?;
        Ok(paths_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_install(
        &mut self,
        install: &NewCollectionElementInstall,
    ) -> anyhow::Result<()> {
        let i = install.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query(
                    "INSERT INTO collection_element_installs (collection_element_id, install_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             install_at = ?, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(i.collection_element_id.value)
                .bind(i.install_at.naive_utc())
                .bind(i.install_at.naive_utc())
                .execute(conn)
                .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_element_install_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementInstall>> {
        let idv = id.value;
        let install_table: Option<CollectionElementInstallTable> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(query_as(
                        "SELECT * FROM collection_element_installs WHERE collection_element_id = ?",
                    )
                    .bind(idv)
                    .fetch_optional(conn)
                    .await?)
                })
            })
            .await?;
        Ok(install_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_play(
        &mut self,
        play: &NewCollectionElementPlay,
    ) -> anyhow::Result<()> {
        let p = play.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query(
                    "INSERT INTO collection_element_plays (collection_element_id, last_play_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             last_play_at = ?, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(p.collection_element_id.value)
                .bind(p.last_play_at.naive_utc())
                .bind(p.last_play_at.naive_utc())
                .execute(conn)
                .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_element_play_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementPlay>> {
        let idv = id.value;
        let play_table: Option<CollectionElementPlayTable> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(query_as(
                        "SELECT * FROM collection_element_plays WHERE collection_element_id = ?",
                    )
                    .bind(idv)
                    .fetch_optional(conn)
                    .await?)
                })
            })
            .await?;
        Ok(play_table.map(|t| t.try_into()).transpose()?)
    }

    async fn update_element_last_play_at_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> anyhow::Result<()> {
        let play = NewCollectionElementPlay::new(id.clone(), last_play_at);
        self.upsert_collection_element_play(&play).await
    }

    async fn upsert_collection_element_like(
        &mut self,
        like: &NewCollectionElementLike,
    ) -> anyhow::Result<()> {
        let l = like.clone();
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query(
                        "INSERT INTO collection_element_likes (collection_element_id, like_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             like_at = ?, updated_at = CURRENT_TIMESTAMP",
                    )
                    .bind(l.collection_element_id.value)
                    .bind(l.like_at.naive_utc())
                    .bind(l.like_at.naive_utc())
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn delete_collection_element_like_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let idv = id.value;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query("DELETE FROM collection_element_likes WHERE collection_element_id = ?")
                        .bind(idv)
                        .execute(conn)
                        .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_element_like_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementLike>> {
        let idv = id.value;
        let like_table: Option<CollectionElementLikeTable> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(query_as(
                        "SELECT * FROM collection_element_likes WHERE collection_element_id = ?",
                    )
                    .bind(idv)
                    .fetch_optional(conn)
                    .await?)
                })
            })
            .await?;
        Ok(like_table.map(|t| t.try_into()).transpose()?)
    }

    async fn update_element_like_at_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        like_at: Option<DateTime<Local>>,
    ) -> anyhow::Result<()> {
        match like_at {
            Some(at) => {
                let like = NewCollectionElementLike::new(id.clone(), at);
                self.upsert_collection_element_like(&like).await
            }
            None => self.delete_collection_element_like_by_element_id(id).await,
        }
    }

    async fn upsert_collection_element_thumbnail(
        &mut self,
        thumbnail: &NewCollectionElementThumbnail,
    ) -> anyhow::Result<()> {
        let t = thumbnail.clone();
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query(
                    "INSERT INTO collection_element_thumbnails (collection_element_id, thumbnail_width, thumbnail_height) 
             VALUES (?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             thumbnail_width = ?, thumbnail_height = ?, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(t.collection_element_id.value)
                .bind(t.thumbnail_width)
                .bind(t.thumbnail_height)
                .bind(t.thumbnail_width)
                .bind(t.thumbnail_height)
                .execute(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn get_element_thumbnail_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementThumbnail>> {
        let idv = id.value;
        let thumbnail_table: Option<CollectionElementThumbnailTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_thumbnails WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(thumbnail_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_thumbnail_size(
        &mut self,
        id: &Id<CollectionElement>,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        let thumbnail = NewCollectionElementThumbnail::new(id.clone(), Some(width), Some(height));
        self.upsert_collection_element_thumbnail(&thumbnail).await
    }

    async fn get_null_thumbnail_size_element_ids(
        &mut self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let ids: Vec<(i32,)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    Ok(sqlx::query_as(
                        "SELECT ce.id 
             FROM collection_elements ce 
             LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
             WHERE cet.collection_element_id IS NULL 
             OR cet.thumbnail_width IS NULL 
             OR cet.thumbnail_height IS NULL",
                    )
                    .fetch_all(conn)
                    .await?)
                })
            })
            .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn get_element_erogamescape_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementErogamescape>> {
        let idv = id.value;
        let row: Option<CollectionElementErogamescapeTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_erogamescape_map WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }

    async fn get_collection_id_by_erogamescape_id(
        &mut self,
        erogamescape_id: i32,
    ) -> anyhow::Result<Option<Id<CollectionElement>>> {
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as("SELECT collection_element_id FROM collection_element_erogamescape_map WHERE erogamescape_id = ?").bind(erogamescape_id).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|v| Id::new(v.0)))
    }

    async fn get_collection_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> anyhow::Result<Vec<(i32, Id<CollectionElement>)>> {
        use sqlx::QueryBuilder;
        if erogamescape_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids = erogamescape_ids.to_vec();
        let rows: Vec<(i32, i32)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let mut qb = QueryBuilder::new(
                        r#"
            SELECT erogamescape_id, collection_element_id
            FROM collection_element_erogamescape_map
            WHERE erogamescape_id IN (
            "#,
                    );
                    {
                        let mut separated = qb.separated(", ");
                        for id in ids.iter() {
                            separated.push_bind(*id);
                        }
                    }
                    qb.push(")");
                    let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|(egs, ce)| (egs, Id::new(ce)))
            .collect())
    }

    async fn upsert_erogamescape_map(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        erogamescape_id: i32,
    ) -> anyhow::Result<()> {
        let ce = collection_element_id.value;
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query(
                    "INSERT INTO collection_element_erogamescape_map (collection_element_id, erogamescape_id) VALUES (?, ?)\n             ON CONFLICT(collection_element_id) DO UPDATE SET erogamescape_id = excluded.erogamescape_id, updated_at = CURRENT_TIMESTAMP",
                )
                .bind(ce)
                .bind(erogamescape_id)
                .execute(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn allocate_new_collection_element_id(
        &mut self,
        gamename: &str,
    ) -> anyhow::Result<Id<CollectionElement>> {
        let gamename = gamename.to_string();
        let id = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let max_id: Option<(i32,)> =
                        sqlx::query_as("SELECT COALESCE(MAX(id), 0) FROM collection_elements")
                            .fetch_optional(conn)
                            .await?;
                    let next_id = max_id.map(|v| v.0).unwrap_or(0) + 1;
                    Ok::<i32, anyhow::Error>(next_id)
                })
            })
            .await?;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    query("INSERT INTO collection_elements (id, gamename) VALUES (?, ?)")
                        .bind(id)
                        .bind(gamename)
                        .execute(conn)
                        .await?;
                    Ok::<i32, anyhow::Error>(id)
                })
            })
            .await?;
        Ok(Id::new(id))
    }

    async fn get_erogamescape_id_by_collection_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<i32>> {
        let idv = id.value;
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as("SELECT erogamescape_id FROM collection_element_erogamescape_map WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|v| v.0))
    }

    async fn get_collection_ids_by_work_ids(
        &mut self,
        work_ids: &[Id<Work>],
    ) -> anyhow::Result<Vec<(Id<Work>, Id<CollectionElement>)>> {
        use sqlx::QueryBuilder;
        if work_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<i32> = work_ids.iter().map(|id| id.value).collect();
        let rows: Vec<(i32, i32)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let mut qb = QueryBuilder::new(
                        r#"
            SELECT m.work_id, m.collection_element_id
            FROM work_collection_elements m
            WHERE m.work_id IN (
            "#,
                    );
                    {
                        let mut separated = qb.separated(", ");
                        for wid in ids.iter() {
                            separated.push_bind(*wid);
                        }
                    }
                    qb.push(")");
                    let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|(wid, ce)| (Id::new(wid), Id::new(ce)))
            .collect())
    }

    async fn upsert_work_mapping(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        work_id: Id<Work>,
    ) -> anyhow::Result<()> {
        let ce = collection_element_id.value;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
            INSERT OR IGNORE INTO work_collection_elements (work_id, collection_element_id)
            VALUES (?, ?)
            "#,
                    )
                    .bind(work_id.value as i64)
                    .bind(ce)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn insert_work_mapping(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        work_id: Id<Work>,
    ) -> anyhow::Result<()> {
        let ce = collection_element_id.value;
        self.executor
            .with_conn(|conn| {
                Box::pin(async move {
                    sqlx::query(
                        r#"
            INSERT INTO work_collection_elements (work_id, collection_element_id)
            VALUES (?, ?)
            "#,
                    )
                    .bind(work_id.value as i64)
                    .bind(ce)
                    .execute(conn)
                    .await?;
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await?;
        Ok(())
    }

    async fn get_work_ids_by_collection_ids(
        &mut self,
        collection_element_ids: &[Id<CollectionElement>],
    ) -> anyhow::Result<Vec<(Id<CollectionElement>, Id<Work>)>> {
        use sqlx::QueryBuilder;
        if collection_element_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<i32> = collection_element_ids.iter().map(|id| id.value).collect();
        let rows: Vec<(i32, i32)> = self
            .executor
            .with_conn(|conn| {
                Box::pin(async move {
                    let mut qb = QueryBuilder::new(
                        r#"
            SELECT m.collection_element_id, m.work_id
            FROM work_collection_elements m
            WHERE m.collection_element_id IN (
            "#,
                    );
                    {
                        let mut separated = qb.separated(", ");
                        for id in ids.iter() {
                            separated.push_bind(*id);
                        }
                    }
                    qb.push(")");
                    let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                    Ok(rows)
                })
            })
            .await?;
        Ok(rows
            .into_iter()
            .map(|(ce, wid)| (Id::new(ce), Id::new(wid)))
            .collect())
    }
}
