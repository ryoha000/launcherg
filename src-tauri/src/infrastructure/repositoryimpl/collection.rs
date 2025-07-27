use async_trait::async_trait;
use chrono::{DateTime, Local};
use sqlx::{query, query_as, Row};

use super::{
    models::collection::{
        CollectionElementInfoTable, CollectionElementInstallTable, CollectionElementLikeTable,
        CollectionElementPathsTable, CollectionElementPlayTable, CollectionElementTable,
        CollectionElementThumbnailTable,
    },
    repository::RepositoryImpl,
};
use crate::domain::{
    collection::{
        CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementLike,
        CollectionElementPaths, CollectionElementPlay, CollectionElementThumbnail,
        NewCollectionElement, NewCollectionElementInfo,
        NewCollectionElementInstall, NewCollectionElementLike, NewCollectionElementPaths,
        NewCollectionElementPlay, NewCollectionElementThumbnail,
    },
    repository::collection::CollectionRepository,
    Id,
};

#[async_trait]
impl CollectionRepository for RepositoryImpl<CollectionElement> {
    // CollectionElement基本操作
    async fn get_all_elements(&self) -> anyhow::Result<Vec<CollectionElement>> {
        let pool = self.pool.0.clone();
        let elements = query_as::<_, CollectionElementTable>("SELECT * FROM collection_elements")
            .fetch_all(&*pool)
            .await?;

        let mut result = Vec::new();
        for element_table in elements {
            let element_id = Id::new(element_table.id);
            let mut element: CollectionElement = element_table.try_into()?;

            // 関連データを取得して設定
            element.info = self.get_element_info_by_element_id(&element_id).await?;
            element.paths = self.get_element_paths_by_element_id(&element_id).await?;
            element.install = self.get_element_install_by_element_id(&element_id).await?;
            element.play = self.get_element_play_by_element_id(&element_id).await?;
            element.like = self.get_element_like_by_element_id(&element_id).await?;
            element.thumbnail = self.get_element_thumbnail_by_element_id(&element_id).await?;

            result.push(element);
        }
        Ok(result)
    }

    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElement>> {
        let pool = self.pool.0.clone();
        
        let row = query(
            "SELECT 
                ce.id, ce.created_at, ce.updated_at,
                cei.id as info_id, cei.gamename, cei.gamename_ruby, cei.sellday, cei.is_nukige, cei.brandname, cei.brandname_ruby, cei.created_at as info_created_at, cei.updated_at as info_updated_at,
                cep.id as paths_id, cep.exe_path, cep.lnk_path, cep.created_at as paths_created_at, cep.updated_at as paths_updated_at,
                cei_install.id as install_id, cei_install.install_at, cei_install.created_at as install_created_at, cei_install.updated_at as install_updated_at,
                cei_play.id as play_id, cei_play.last_play_at, cei_play.created_at as play_created_at, cei_play.updated_at as play_updated_at,
                cei_like.id as like_id, cei_like.like_at, cei_like.created_at as like_created_at, cei_like.updated_at as like_updated_at,
                cet.id as thumbnail_id, cet.thumbnail_width, cet.thumbnail_height, cet.created_at as thumbnail_created_at, cet.updated_at as thumbnail_updated_at
            FROM collection_elements ce
            LEFT JOIN collection_element_info_by_erogamescape cei ON ce.id = cei.collection_element_id
            LEFT JOIN collection_element_paths cep ON ce.id = cep.collection_element_id
            LEFT JOIN collection_element_installs cei_install ON ce.id = cei_install.collection_element_id
            LEFT JOIN collection_element_plays cei_play ON ce.id = cei_play.collection_element_id
            LEFT JOIN collection_element_likes cei_like ON ce.id = cei_like.collection_element_id
            LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
            WHERE ce.id = ?"
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        let row = match row {
            Some(row) => row,
            None => return Ok(None),
        };

        // CollectionElementを構築
        let element_table = CollectionElementTable {
            id: row.get("id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let mut element: CollectionElement = element_table.try_into()?;

        // 関連データを設定
        element.info = if let Some(info_id) = row.get::<Option<i32>, _>("info_id") {
            Some(CollectionElementInfo::new(
                Id::new(info_id),
                id.clone(),
                row.get("gamename"),
                row.get("gamename_ruby"),
                row.get("brandname"),
                row.get("brandname_ruby"),
                row.get("sellday"),
                row.get::<i32, _>("is_nukige") != 0,
                row.get::<chrono::NaiveDateTime, _>("info_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("info_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        element.paths = if let Some(paths_id) = row.get::<Option<i32>, _>("paths_id") {
            Some(CollectionElementPaths::new(
                Id::new(paths_id),
                id.clone(),
                row.get("exe_path"),
                row.get("lnk_path"),
                row.get::<chrono::NaiveDateTime, _>("paths_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("paths_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        element.install = if let Some(install_id) = row.get::<Option<i32>, _>("install_id") {
            Some(CollectionElementInstall::new(
                Id::new(install_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("install_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("install_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("install_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        element.play = if let Some(play_id) = row.get::<Option<i32>, _>("play_id") {
            Some(CollectionElementPlay::new(
                Id::new(play_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("last_play_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("play_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("play_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        element.like = if let Some(like_id) = row.get::<Option<i32>, _>("like_id") {
            Some(CollectionElementLike::new(
                Id::new(like_id),
                id.clone(),
                row.get::<chrono::NaiveDateTime, _>("like_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("like_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("like_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        element.thumbnail = if let Some(thumbnail_id) = row.get::<Option<i32>, _>("thumbnail_id") {
            Some(CollectionElementThumbnail::new(
                Id::new(thumbnail_id),
                id.clone(),
                row.get("thumbnail_width"),
                row.get("thumbnail_height"),
                row.get::<chrono::NaiveDateTime, _>("thumbnail_created_at").and_utc().with_timezone(&chrono::Local),
                row.get::<chrono::NaiveDateTime, _>("thumbnail_updated_at").and_utc().with_timezone(&chrono::Local),
            ))
        } else {
            None
        };

        Ok(Some(element))
    }

    async fn upsert_collection_element(&self, new_element: &NewCollectionElement) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_elements (id) VALUES (?) 
             ON CONFLICT(id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP",
        )
        .bind(new_element.id.value)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("DELETE FROM collection_elements WHERE id = ?")
            .bind(element_id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn delete_element_by_id(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        self.delete_collection_element(id).await
    }

    // CollectionElementInfo操作
    async fn upsert_collection_element_info(&self, info: &NewCollectionElementInfo) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_info_by_erogamescape 
             (collection_element_id, gamename, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby) 
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             gamename = ?, gamename_ruby = ?, sellday = ?, is_nukige = ?, 
             brandname = ?, brandname_ruby = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(info.collection_element_id.value)
        .bind(&info.gamename)
        .bind(&info.gamename_ruby)
        .bind(&info.sellday)
        .bind(if info.is_nukige { 1 } else { 0 })
        .bind(&info.brandname)
        .bind(&info.brandname_ruby)
        .bind(&info.gamename)
        .bind(&info.gamename_ruby)
        .bind(&info.sellday)
        .bind(if info.is_nukige { 1 } else { 0 })
        .bind(&info.brandname)
        .bind(&info.brandname_ruby)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_element_info_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementInfo>> {
        let pool = self.pool.0.clone();
        let info_table = query_as::<_, CollectionElementInfoTable>(
            "SELECT * FROM collection_element_info_by_erogamescape WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match info_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }
    

    async fn get_not_registered_info_element_ids(&self) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT ce.id
            FROM collection_elements ce
            LEFT JOIN collection_element_info_by_erogamescape cei
            ON ce.id = cei.collection_element_id
            WHERE cei.collection_element_id IS NULL",
        )
        .fetch_all(&*pool)
        .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn get_brandname_and_rubies(&self) -> anyhow::Result<Vec<(String, String)>> {
        let pool = self.pool.0.clone();
        Ok(sqlx::query_as(
            "SELECT DISTINCT brandname, brandname_ruby FROM collection_element_info_by_erogamescape",
        )
        .fetch_all(&*pool)
        .await?)
    }

    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = match is_nukige {
            true => sqlx::query_as(
                "SELECT collection_element_id FROM collection_element_info_by_erogamescape WHERE is_nukige != 0",
            ),
            false => sqlx::query_as(
                "SELECT collection_element_id FROM collection_element_info_by_erogamescape WHERE is_nukige = 0",
            ),
        }
        .fetch_all(&*pool)
        .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let mut builder = sqlx::query_builder::QueryBuilder::new(
            "SELECT collection_element_id FROM collection_element_info_by_erogamescape WHERE brandname IN (",
        );
        let mut separated = builder.separated(", ");
        for name in brandnames.iter() {
            separated.push_bind(name);
        }
        separated.push_unseparated(")");
        let query = builder.build();
        let ids: Vec<i32> = query
            .fetch_all(&*pool)
            .await?
            .into_iter()
            .map(|v| v.try_get(0))
            .filter_map(|v| v.ok())
            .collect();

        Ok(ids.into_iter().map(|v| Id::new(v)).collect())
    }

    async fn get_element_ids_by_sellday(
        &self,
        since: &str,
        until: &str,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT collection_element_id FROM collection_element_info_by_erogamescape 
             WHERE DATE(sellday) BETWEEN ? AND ?",
        )
        .bind(since)
        .bind(until)
        .fetch_all(&*pool)
        .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    // CollectionElementPaths操作
    async fn upsert_collection_element_paths(&self, paths: &NewCollectionElementPaths) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_paths (collection_element_id, exe_path, lnk_path) 
             VALUES (?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             exe_path = ?, lnk_path = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(paths.collection_element_id.value)
        .bind(&paths.exe_path)
        .bind(&paths.lnk_path)
        .bind(&paths.exe_path)
        .bind(&paths.lnk_path)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_element_paths_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementPaths>> {
        let pool = self.pool.0.clone();
        let paths_table = query_as::<_, CollectionElementPathsTable>(
            "SELECT * FROM collection_element_paths WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match paths_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }

    // CollectionElementInstall操作
    async fn upsert_collection_element_install(
        &self,
        install: &NewCollectionElementInstall,
    ) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_installs (collection_element_id, install_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             install_at = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(install.collection_element_id.value)
        .bind(install.install_at.naive_utc())
        .bind(install.install_at.naive_utc())
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_element_install_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementInstall>> {
        let pool = self.pool.0.clone();
        let install_table = query_as::<_, CollectionElementInstallTable>(
            "SELECT * FROM collection_element_installs WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match install_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }

    async fn get_element_ids_by_install_at_not_null(&self) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT collection_element_id FROM collection_element_installs"
        )
        .fetch_all(&*pool)
        .await?;

        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    // CollectionElementPlay操作
    async fn upsert_collection_element_play(&self, play: &NewCollectionElementPlay) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_plays (collection_element_id, last_play_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             last_play_at = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(play.collection_element_id.value)
        .bind(play.last_play_at.naive_utc())
        .bind(play.last_play_at.naive_utc())
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_element_play_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementPlay>> {
        let pool = self.pool.0.clone();
        let play_table = query_as::<_, CollectionElementPlayTable>(
            "SELECT * FROM collection_element_plays WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match play_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update_element_last_play_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> anyhow::Result<()> {
        let play = NewCollectionElementPlay::new(id.clone(), last_play_at);
        self.upsert_collection_element_play(&play).await
    }

    // CollectionElementLike操作
    async fn upsert_collection_element_like(&self, like: &NewCollectionElementLike) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_likes (collection_element_id, like_at) 
             VALUES (?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             like_at = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(like.collection_element_id.value)
        .bind(like.like_at.naive_utc())
        .bind(like.like_at.naive_utc())
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn delete_collection_element_like_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query("DELETE FROM collection_element_likes WHERE collection_element_id = ?")
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn get_element_like_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementLike>> {
        let pool = self.pool.0.clone();
        let like_table = query_as::<_, CollectionElementLikeTable>(
            "SELECT * FROM collection_element_likes WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match like_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update_element_like_at_by_id(
        &self,
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

    // CollectionElementThumbnail操作
    async fn upsert_collection_element_thumbnail(
        &self,
        thumbnail: &NewCollectionElementThumbnail,
    ) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        query(
            "INSERT INTO collection_element_thumbnails (collection_element_id, thumbnail_width, thumbnail_height) 
             VALUES (?, ?, ?)
             ON CONFLICT(collection_element_id) DO UPDATE SET 
             thumbnail_width = ?, thumbnail_height = ?, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(thumbnail.collection_element_id.value)
        .bind(thumbnail.thumbnail_width)
        .bind(thumbnail.thumbnail_height)
        .bind(thumbnail.thumbnail_width)
        .bind(thumbnail.thumbnail_height)
        .execute(&*pool)
        .await?;
        Ok(())
    }

    async fn get_element_thumbnail_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<CollectionElementThumbnail>> {
        let pool = self.pool.0.clone();
        let thumbnail_table = query_as::<_, CollectionElementThumbnailTable>(
            "SELECT * FROM collection_element_thumbnails WHERE collection_element_id = ?",
        )
        .bind(id.value)
        .fetch_optional(&*pool)
        .await?;

        match thumbnail_table {
            Some(table) => Ok(Some(table.try_into()?)),
            None => Ok(None),
        }
    }

    async fn upsert_collection_element_thumbnail_size(
        &self,
        id: &Id<CollectionElement>,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        let thumbnail = NewCollectionElementThumbnail::new(id.clone(), Some(width), Some(height));
        self.upsert_collection_element_thumbnail(&thumbnail).await
    }

    async fn get_null_thumbnail_size_element_ids(&self) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT ce.id 
             FROM collection_elements ce 
             LEFT JOIN collection_element_thumbnails cet ON ce.id = cet.collection_element_id
             WHERE cet.collection_element_id IS NULL 
             OR cet.thumbnail_width IS NULL 
             OR cet.thumbnail_height IS NULL",
        )
        .fetch_all(&*pool)
        .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }

    // その他のユーティリティ操作
    async fn remove_conflict_maps(&self) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        let not_delete_ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT MIN(id) FROM collection_element_maps GROUP BY collection_id, collection_element_id",
        )
        .fetch_all(&*pool)
        .await?;
        let not_delete_ids: Vec<i32> = not_delete_ids.into_iter().map(|v| v.0).collect();

        if not_delete_ids.is_empty() {
            return Ok(());
        }
        let mut builder = sqlx::query_builder::QueryBuilder::new(
            "DELETE FROM collection_element_maps WHERE id NOT IN (",
        );
        let mut separated = builder.separated(", ");
        for id in not_delete_ids.iter() {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        let query = builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }

}