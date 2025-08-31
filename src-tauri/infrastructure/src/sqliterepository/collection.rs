use chrono::{DateTime, Local};
use domain::{
    collection::{
        CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementLike,
        CollectionElementPaths, CollectionElementPlay, CollectionElementThumbnail,
        CollectionElementErogamescape, NewCollectionElement, NewCollectionElementInfo,
        NewCollectionElementInstall, NewCollectionElementLike, NewCollectionElementPaths,
        NewCollectionElementPlay, NewCollectionElementThumbnail,
    },
    Id,
};
use domain::repository::collection::CollectionRepository;
use sqlx::{query, query_as, Row};

use crate::sqliterepository::models::collection::{
    CollectionElementTable, CollectionElementInfoTable, CollectionElementInstallTable,
    CollectionElementLikeTable, CollectionElementPathsTable, CollectionElementPlayTable,
    CollectionElementThumbnailTable, CollectionElementErogamescapeTable,
};
use crate::sqliterepository::sqliterepository::RepositoryImpl;

impl CollectionRepository for RepositoryImpl<domain::collection::CollectionElement> {
    async fn get_all_elements(&mut self) -> anyhow::Result<Vec<CollectionElement>> {
        // 旧実装はリッチ JOIN を各要素ごとに追っていた。まずは elements の一覧のみ取得し、詳細は既存 API で埋める。
        let rows: Vec<CollectionElementTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_elements").fetch_all(conn).await?) })
        }).await?;
        let mut result = Vec::new();
        for element_table in rows {
            let element_id = Id::new(element_table.id);
            let mut element: CollectionElement = element_table.try_into()?;

            element.info = self.get_element_info_by_element_id(&element_id).await?;
            element.paths = self.get_element_paths_by_element_id(&element_id).await?;
            element.install = self.get_element_install_by_element_id(&element_id).await?;
            element.play = self.get_element_play_by_element_id(&element_id).await?;
            element.like = self.get_element_like_by_element_id(&element_id).await?;
            element.thumbnail = self.get_element_thumbnail_by_element_id(&element_id).await?;
            element.erogamescape = self.get_element_erogamescape_by_element_id(&element_id).await?;

            result.push(element);
        }
        Ok(result)
    }

    async fn get_element_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElement>> {
        let idv = id.value;
        let row = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(query(
                    "SELECT 
                        ce.id, ce.gamename, ce.created_at, ce.updated_at,
                        cei.id as info_id, cei.gamename_ruby, cei.sellday, cei.is_nukige, cei.brandname, cei.brandname_ruby, cei.created_at as info_created_at, cei.updated_at as info_updated_at,
                        cep.id as paths_id, cep.exe_path, cep.lnk_path, cep.created_at as paths_created_at, cep.updated_at as paths_updated_at,
                        cei_install.id as install_id, cei_install.install_at, cei_install.created_at as install_created_at, cei_install.updated_at as install_updated_at,
                        cei_play.id as play_id, cei_play.last_play_at, cei_play.created_at as play_created_at, cei_play.updated_at as play_updated_at,
                        cei_like.id as like_id, cei_like.like_at, cei_like.created_at as like_created_at, cei_like.updated_at as like_updated_at,
                        cet.id as thumbnail_id, cet.thumbnail_width, cet.thumbnail_height, cet.created_at as thumbnail_created_at, cet.updated_at as thumbnail_updated_at,
                        cee.id as egs_id, cee.erogamescape_id as egs_erogamescape_id, cee.created_at as egs_created_at, cee.updated_at as egs_updated_at
                    FROM collection_elements ce
                    LEFT JOIN collection_element_info_by_erogamescape cei ON ce.id = cei.collection_element_id
                    LEFT JOIN collection_element_paths cep ON ce.id = cep.collection_element_id
                    LEFT JOIN collection_element_installs cei_install ON ce.id = cei_install.collection_element_id
                    LEFT JOIN collection_element_plays cei_play ON ce.id = cei_play.collection_element_id
                    LEFT JOIN collection_element_likes cei_like ON ce.id = cei_like.collection_element_id
                    LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
                    LEFT JOIN collection_element_erogamescape_map cee ON ce.id = cee.collection_element_id
                    WHERE ce.id = ?"
                )
                .bind(idv)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;

        let row = match row { Some(row) => row, None => return Ok(None) };

        let element_table = CollectionElementTable {
            id: row.get("id"),
            gamename: row.get("gamename"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let mut element: CollectionElement = element_table.try_into()?;

        element.info = if let Some(info_id) = row.get::<Option<i32>, _>("info_id") {
            Some(CollectionElementInfo::new(
                Id::new(info_id),
                id.clone(),
                row.get("gamename_ruby"),
                row.get("brandname"),
                row.get("brandname_ruby"),
                row.get("sellday"),
                row.get::<i32, _>("is_nukige") != 0,
                row.get::<chrono::NaiveDateTime, _>("info_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("info_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.paths = if let Some(paths_id) = row.get::<Option<i32>, _>("paths_id") {
            Some(CollectionElementPaths::new(
                Id::new(paths_id),
                id.clone(),
                row.get("exe_path"),
                row.get("lnk_path"),
                row.get::<chrono::NaiveDateTime, _>("paths_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("paths_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.install = if let Some(install_id) = row.get::<Option<i32>, _>("install_id") {
            Some(CollectionElementInstall::new(
                Id::new(install_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("install_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("install_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("install_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.play = if let Some(play_id) = row.get::<Option<i32>, _>("play_id") {
            Some(CollectionElementPlay::new(
                Id::new(play_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("last_play_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("play_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("play_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.like = if let Some(like_id) = row.get::<Option<i32>, _>("like_id") {
            Some(CollectionElementLike::new(
                Id::new(like_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("like_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("like_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("like_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.thumbnail = if let Some(thumbnail_id) = row.get::<Option<i32>, _>("thumbnail_id") {
            Some(CollectionElementThumbnail::new(
                Id::new(thumbnail_id),
                id.clone(),
                row.get("thumbnail_width"),
                row.get("thumbnail_height"),
                row.get::<chrono::NaiveDateTime, _>("thumbnail_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("thumbnail_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        element.erogamescape = if let Some(egs_row_id) = row.get::<Option<i32>, _>("egs_id") {
            Some(domain::collection::CollectionElementErogamescape::new(
                Id::new(egs_row_id),
                id.clone(),
                row.get("egs_erogamescape_id"),
                row.get::<chrono::NaiveDateTime, _>("egs_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("egs_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else { None };

        Ok(Some(element))
    }

    async fn upsert_collection_element(&mut self, new_element: &NewCollectionElement) -> anyhow::Result<()> {
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

    async fn delete_collection_element(&mut self, element_id: &Id<CollectionElement>) -> anyhow::Result<()> {
        let idv = element_id.value;
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query("DELETE FROM collection_elements WHERE id = ?").bind(idv).execute(conn).await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn upsert_collection_element_info(&mut self, info: &NewCollectionElementInfo) -> anyhow::Result<()> {
        let i = info.clone();
        self.executor.with_conn(|conn| {
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
        }).await?;
        Ok(())
    }

    async fn get_element_info_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementInfo>> {
        let idv = id.value;
        let info_table: Option<CollectionElementInfoTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_info_by_erogamescape WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(info_table.map(|t| t.try_into()).transpose()?)
    }

    async fn get_not_registered_info_element_ids(&mut self) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let ids: Vec<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(
                "SELECT ce.id
            FROM collection_elements ce
            LEFT JOIN collection_element_info_by_erogamescape cei
            ON ce.id = cei.collection_element_id
            WHERE cei.collection_element_id IS NULL",
            ).fetch_all(conn).await?) })
        }).await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn upsert_collection_element_paths(&mut self, paths: &NewCollectionElementPaths) -> anyhow::Result<()> {
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

    async fn get_element_paths_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementPaths>> {
        let idv = id.value;
        let paths_table: Option<CollectionElementPathsTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_paths WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(paths_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_install(&mut self, install: &NewCollectionElementInstall) -> anyhow::Result<()> {
        let i = install.clone();
        self.executor.with_conn(|conn| {
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
        }).await?;
        Ok(())
    }

    async fn get_element_install_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementInstall>> {
        let idv = id.value;
        let install_table: Option<CollectionElementInstallTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_installs WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(install_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_play(&mut self, play: &NewCollectionElementPlay) -> anyhow::Result<()> {
        let p = play.clone();
        self.executor.with_conn(|conn| {
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
        }).await?;
        Ok(())
    }

    async fn get_element_play_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementPlay>> {
        let idv = id.value;
        let play_table: Option<CollectionElementPlayTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_plays WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(play_table.map(|t| t.try_into()).transpose()?)
    }

    async fn update_element_last_play_at_by_id(&mut self, id: &Id<CollectionElement>, last_play_at: DateTime<Local>) -> anyhow::Result<()> {
        let play = NewCollectionElementPlay::new(id.clone(), last_play_at);
        self.upsert_collection_element_play(&play).await
    }

    async fn upsert_collection_element_like(&mut self, like: &NewCollectionElementLike) -> anyhow::Result<()> {
        let l = like.clone();
        self.executor.with_conn(|conn| {
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
        }).await?;
        Ok(())
    }

    async fn delete_collection_element_like_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        let idv = id.value;
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query("DELETE FROM collection_element_likes WHERE collection_element_id = ?").bind(idv).execute(conn).await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn get_element_like_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementLike>> {
        let idv = id.value;
        let like_table: Option<CollectionElementLikeTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_likes WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(like_table.map(|t| t.try_into()).transpose()?)
    }

    async fn update_element_like_at_by_id(&mut self, id: &Id<CollectionElement>, like_at: Option<DateTime<Local>>) -> anyhow::Result<()> {
        match like_at {
            Some(at) => {
                let like = NewCollectionElementLike::new(id.clone(), at);
                self.upsert_collection_element_like(&like).await
            }
            None => self.delete_collection_element_like_by_element_id(id).await,
        }
    }

    async fn upsert_collection_element_thumbnail(&mut self, thumbnail: &NewCollectionElementThumbnail) -> anyhow::Result<()> {
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

    async fn get_element_thumbnail_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementThumbnail>> {
        let idv = id.value;
        let thumbnail_table: Option<CollectionElementThumbnailTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_thumbnails WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(thumbnail_table.map(|t| t.try_into()).transpose()?)
    }

    async fn upsert_collection_element_thumbnail_size(&mut self, id: &Id<CollectionElement>, width: i32, height: i32) -> anyhow::Result<()> {
        let thumbnail = NewCollectionElementThumbnail::new(id.clone(), Some(width), Some(height));
        self.upsert_collection_element_thumbnail(&thumbnail).await
    }

    async fn get_null_thumbnail_size_element_ids(&mut self) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let ids: Vec<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as(
                "SELECT ce.id 
             FROM collection_elements ce 
             LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
             WHERE cet.collection_element_id IS NULL 
             OR cet.thumbnail_width IS NULL 
             OR cet.thumbnail_height IS NULL",
            ).fetch_all(conn).await?) })
        }).await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn get_element_erogamescape_by_element_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<CollectionElementErogamescape>> {
        let idv = id.value;
        let row: Option<CollectionElementErogamescapeTable> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(query_as("SELECT * FROM collection_element_erogamescape_map WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|t| t.try_into()).transpose()?)
    }

    async fn get_collection_id_by_erogamescape_id(&mut self, erogamescape_id: i32) -> anyhow::Result<Option<Id<CollectionElement>>> {
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as("SELECT collection_element_id FROM collection_element_erogamescape_map WHERE erogamescape_id = ?").bind(erogamescape_id).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|v| Id::new(v.0)))
    }

    async fn get_collection_ids_by_erogamescape_ids(&mut self, erogamescape_ids: &[i32]) -> anyhow::Result<Vec<(i32, Id<CollectionElement>)>> {
        use sqlx::QueryBuilder;
        if erogamescape_ids.is_empty() { return Ok(Vec::new()); }
        let ids = erogamescape_ids.to_vec();
        let rows: Vec<(i32, i32)> = self.executor.with_conn(|conn| {
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
                    for id in ids.iter() { separated.push_bind(*id); }
                }
                qb.push(")");
                let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|(egs, ce)| (egs, Id::new(ce))).collect())
    }

    async fn upsert_erogamescape_map(&mut self, collection_element_id: &Id<CollectionElement>, erogamescape_id: i32) -> anyhow::Result<()> {
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

    async fn allocate_new_collection_element_id(&mut self, gamename: &str) -> anyhow::Result<Id<CollectionElement>> {
        let gamename = gamename.to_string();
        let id = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let max_id: Option<(i32,)> = sqlx::query_as("SELECT COALESCE(MAX(id), 0) FROM collection_elements")
                    .fetch_optional(conn)
                    .await?;
                let next_id = max_id.map(|v| v.0).unwrap_or(0) + 1;
                Ok::<i32, anyhow::Error>(next_id)
            })
        }).await?;
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                query("INSERT INTO collection_elements (id, gamename) VALUES (?, ?)")
                    .bind(id)
                    .bind(gamename)
                    .execute(conn)
                    .await?;
                Ok::<i32, anyhow::Error>(id)
            })
        }).await?;
        Ok(Id::new(id))
    }

    async fn get_erogamescape_id_by_collection_id(&mut self, id: &Id<CollectionElement>) -> anyhow::Result<Option<i32>> {
        let idv = id.value;
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as("SELECT erogamescape_id FROM collection_element_erogamescape_map WHERE collection_element_id = ?").bind(idv).fetch_optional(conn).await?) })
        }).await?;
        Ok(row.map(|v| v.0))
    }

    async fn get_collection_id_by_dmm_mapping(&mut self, store_id: &str, category: &str, subcategory: &str) -> anyhow::Result<Option<Id<CollectionElement>>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let subcategory = subcategory.to_string();
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(
                    r#"
            SELECT m.collection_element_id
            FROM dmm_works w
            JOIN work_collection_elements m ON m.work_id = w.work_id
            WHERE w.store_id = ? AND w.category = ? AND COALESCE(w.subcategory, '') = COALESCE(?, '')
            LIMIT 1
            "#,
                )
                .bind(store_id)
                .bind(category)
                .bind(subcategory)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;
        Ok(row.map(|v| Id::new(v.0)))
    }

    async fn get_collection_ids_by_dmm_mappings(&mut self, keys: &[(String, String, String)]) -> anyhow::Result<Vec<(String, String, String, Id<CollectionElement>)>> {
        use sqlx::QueryBuilder;
        if keys.is_empty() { return Ok(Vec::new()); }
        let keys = keys.to_vec();
        let rows: Vec<(String, String, String, i32)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut qb = QueryBuilder::new(
                    r#"
            SELECT w.store_id, w.category, COALESCE(w.subcategory, '') as subcategory, m.collection_element_id
            FROM dmm_works w
            JOIN work_collection_elements m ON m.work_id = w.work_id
            WHERE (w.store_id, w.category, COALESCE(w.subcategory, '')) IN (
            "#,
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
                let rows: Vec<(String, String, String, i32)> = qb.build_query_as().fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|(sid, cat, sub, ce)| (sid, cat, sub, Id::new(ce))).collect())
    }

    async fn get_collection_ids_by_work_ids(&mut self, work_ids: &[i32]) -> anyhow::Result<Vec<(i32, Id<CollectionElement>)>> {
        use sqlx::QueryBuilder;
        if work_ids.is_empty() { return Ok(Vec::new()); }
        let ids = work_ids.to_vec();
        let rows: Vec<(i32, i32)> = self.executor.with_conn(|conn| {
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
                    for wid in ids.iter() { separated.push_bind(*wid); }
                }
                qb.push(")");
                let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|(wid, ce)| (wid, Id::new(ce))).collect())
    }

    async fn get_collection_id_by_dlsite_mapping(&mut self, store_id: &str, category: &str) -> anyhow::Result<Option<Id<CollectionElement>>> {
        let store_id = store_id.to_string();
        let category = category.to_string();
        let row: Option<(i32,)> = self.executor.with_conn(|conn| {
            Box::pin(async move {
                Ok(sqlx::query_as(
                    r#"
            SELECT m.collection_element_id
            FROM dlsite_works w
            JOIN work_collection_elements m ON m.work_id = w.work_id
            WHERE w.store_id = ? AND w.category = ?
            LIMIT 1
            "#,
                )
                .bind(store_id)
                .bind(category)
                .fetch_optional(conn)
                .await?)
            })
        }).await?;
        Ok(row.map(|v| Id::new(v.0)))
    }

    async fn upsert_work_mapping(&mut self, collection_element_id: &Id<CollectionElement>, work_id: i32) -> anyhow::Result<()> {
        let ce = collection_element_id.value;
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                sqlx::query(
                    r#"
            INSERT OR IGNORE INTO work_collection_elements (work_id, collection_element_id)
            VALUES (?, ?)
            "#,
                )
                .bind(work_id as i64)
                .bind(ce)
                .execute(conn)
                .await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }

    async fn get_work_ids_by_collection_ids(&mut self, collection_element_ids: &[i32]) -> anyhow::Result<Vec<(Id<CollectionElement>, i32)>> {
        use sqlx::QueryBuilder;
        if collection_element_ids.is_empty() { return Ok(Vec::new()); }
        let ids = collection_element_ids.to_vec();
        let rows: Vec<(i32, i32)> = self.executor.with_conn(|conn| {
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
                    for id in ids.iter() { separated.push_bind(*id); }
                }
                qb.push(")");
                let rows: Vec<(i32, i32)> = qb.build_query_as().fetch_all(conn).await?;
                Ok(rows)
            })
        }).await?;
        Ok(rows.into_iter().map(|(ce, wid)| (Id::new(ce), wid)).collect())
    }
}


