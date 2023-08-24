use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use sqlx::{query, query_as, QueryBuilder, Row};

use super::{
    models::collection::{CollectionElementTable, CollectionTable},
    repository::RepositoryImpl,
};
use crate::domain::{
    collection::{
        Collection, CollectionElement, NewCollection, NewCollectionElement,
        NewCollectionElementDetail, UpdateCollection,
    },
    repository::collection::CollectionRepository,
    Id,
};

#[async_trait]
impl CollectionRepository for RepositoryImpl<Collection> {
    async fn get(&self, id: &Id<Collection>) -> Result<Option<Collection>> {
        let pool = self.pool.0.clone();
        let collection = query_as::<_, CollectionTable>("select * from collections where id = ?")
            .bind(id.value)
            .fetch_all(&*pool)
            .await?
            .into_iter()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Collection>>();
        if collection.len() == 0 {
            Ok(None)
        } else {
            let v = collection[0].clone();
            Ok(Some(v))
        }
    }
    async fn get_by_name(&self, name: String) -> Result<Option<Collection>> {
        let pool = self.pool.0.clone();
        let collection = query_as::<_, CollectionTable>("select * from collections where name = ?")
            .bind(name)
            .fetch_all(&*pool)
            .await?
            .into_iter()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Collection>>();
        if collection.len() == 0 {
            Ok(None)
        } else {
            let v = collection[0].clone();
            Ok(Some(v))
        }
    }
    async fn get_all(&self) -> Result<Vec<Collection>> {
        let pool = self.pool.0.clone();
        Ok(query_as::<_, CollectionTable>("select * from collections")
            .fetch_all(&*pool)
            .await?
            .into_iter()
            .filter_map(|v| v.try_into().ok())
            .collect())
    }
    async fn get_all_elements(&self) -> Result<Vec<CollectionElement>> {
        let pool = self.pool.0.clone();
        Ok(query_as::<_, CollectionElementTable>(
            "select ce.*, cde.collection_element_id, cde.gamename_ruby, cde.sellday, cde.is_nukige, cde.brandname, cde.brandname_ruby from collection_elements ce inner join collection_element_details cde on cde.collection_element_id = ce.id",
        )
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .filter_map(|v| v.try_into().ok())
        .collect())
    }
    async fn get_elements_by_id(&self, id: &Id<Collection>) -> Result<Vec<CollectionElement>> {
        let pool = self.pool.0.clone();
        Ok(query_as::<_, CollectionElementTable>(
            "select collection_elements.*, cde.collection_element_id, cde.gamename_ruby, cde.sellday, cde.is_nukige, cde.brandname, cde.brandname_ruby from collection_element_maps cem inner join collection_elements on collection_elements.id = cem.collection_element_id inner join collection_element_details cde on cde.collection_element_id = collection_elements.id where collection_id = ?",
        )
        .bind(id.value)
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .filter_map(|v| v.try_into().ok())
        .collect())
    }
    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>> {
        let pool = self.pool.0.clone();
        let elements =
            query_as::<_, CollectionElementTable>("select collection_elements.*, cde.collection_element_id, cde.gamename_ruby, cde.sellday, cde.is_nukige, cde.brandname, cde.brandname_ruby from collection_elements inner join collection_element_details cde on cde.collection_element_id = collection_elements.id where collection_elements.id = ?")
                .bind(id.value)
                .fetch_all(&*pool)
                .await?
                .into_iter()
                .filter_map(|v| v.try_into().ok())
                .collect::<Vec<CollectionElement>>();
        if elements.len() == 0 {
            Ok(None)
        } else {
            let v = elements[0].clone();
            Ok(Some(v))
        }
    }
    async fn create(&self, new: NewCollection) -> Result<Collection> {
        if new.name.len() == 0 {
            anyhow::bail!("name is required");
        }

        let pool = self.pool.0.clone();
        let id = query("insert into collections (name) values (?)")
            .bind(new.name)
            .execute(&*pool)
            .await?
            .last_insert_rowid();
        Ok(
            query_as::<_, CollectionTable>("select * from collections where id = ?")
                .bind(id)
                .fetch_one(&*pool)
                .await?
                .try_into()?,
        )
    }
    async fn update(&self, src: UpdateCollection) -> Result<()> {
        let pool = self.pool.0.clone();
        query("update collections set name = ? where id = ?")
            .bind(src.name)
            .bind(src.id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn delete(&self, id: &Id<Collection>) -> Result<()> {
        let pool = self.pool.0.clone();
        query("delete from collections where id = ?")
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn upsert_collection_element(&self, new: &NewCollectionElement) -> Result<()> {
        let pool = self.pool.0.clone();
        let _ = query("insert into collection_elements (id, gamename, path, install_at) values (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET gamename = ?, path = ?, install_at = ?, updated_at = ?")
            .bind(new.id.value)
            .bind(new.gamename.clone())
            .bind(new.path.clone())
            .bind(new.install_at.and_then(|v| Some(v.naive_utc()))) // TODO: naive_utc いるか確認
            .bind(new.gamename.clone())
            .bind(new.path.clone())
            .bind(new.install_at.and_then(|v| Some(v.naive_utc()))) // TODO: naive_utc いるか確認
            .bind(Local::now().naive_utc()) // TODO: naive_utc いるか確認
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn create_collection_elements(
        &self,
        new_elements: Vec<NewCollectionElement>,
    ) -> Result<()> {
        if new_elements.len() == 0 {
            return Ok(());
        }
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder =
            QueryBuilder::new("INSERT INTO collection_elements (id, gamename, path, install_at) ");
        query_builder.push_values(new_elements, |mut b, new| {
            b.push_bind(new.id.value)
                .push_bind(new.gamename)
                .push_bind(new.path)
                .push_bind(new.install_at.and_then(|v| Some(v.naive_utc())));
        });

        let pool = self.pool.0.clone();
        let query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn add_elements_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> Result<()> {
        if collection_element_ids.len() == 0 {
            return Ok(());
        }
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO collection_element_maps (collection_id, collection_element_id) ",
        );
        query_builder.push_values(collection_element_ids, |mut b, new| {
            b.push_bind(collection_id.value).push_bind(new.value);
        });

        let pool = self.pool.0.clone();
        let query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn remove_elements_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> Result<()> {
        let pool = self.pool.0.clone();
        let mut builder = sqlx::query_builder::QueryBuilder::new(
            "DELETE FROM collection_element_maps WHERE collection_id = ",
        );
        builder.push_bind(collection_id.value);
        builder.push(" AND collection_element_id IN ( ");
        let mut separated = builder.separated(", ");
        for id in collection_element_ids.iter() {
            separated.push_bind(id.value);
        }
        separated.push_unseparated(")");
        let query = builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn remove_conflict_maps(&self) -> anyhow::Result<()> {
        let pool = self.pool.0.clone();
        let not_delete_ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT MIN(id) FROM collection_element_maps GROUP BY collection_id, collection_element_id",
        )
        .fetch_all(&*pool)
        .await?;
        let not_delete_ids: Vec<i32> = not_delete_ids.into_iter().map(|v| v.0).collect();

        if not_delete_ids.len() == 0 {
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
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()> {
        let pool = self.pool.0.clone();
        query("delete from collection_elements where id = ?")
            .bind(element_id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }

    async fn get_not_registered_detail_element_ids(&self) -> Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let not_registered_ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT ce.id
            FROM collection_elements ce
            LEFT JOIN collection_element_details ced
            ON ce.id = ced.collection_element_id
            WHERE ced.collection_element_id IS NULL",
        )
        .fetch_all(&*pool)
        .await?;
        Ok(not_registered_ids
            .into_iter()
            .map(|v| Id::new(v.0))
            .collect())
    }
    async fn create_element_details(&self, details: Vec<NewCollectionElementDetail>) -> Result<()> {
        if details.len() == 0 {
            return Ok(());
        }
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO collection_element_details (collection_element_id, gamename_ruby, sellday, is_nukige, brandname, brandname_ruby) ",
        );
        query_builder.push_values(details, |mut b, new| {
            let is_nukige = match new.is_nukige {
                true => 1,
                false => 0,
            };
            b.push_bind(new.collection_element_id.value)
                .push_bind(new.gamename_ruby)
                .push_bind(new.sellday)
                .push_bind(is_nukige)
                .push_bind(new.brandname)
                .push_bind(new.brandname_ruby);
        });

        let pool = self.pool.0.clone();
        let query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn get_brandname_and_rubies(&self) -> Result<Vec<(String, String)>> {
        let pool = self.pool.0.clone();
        Ok(sqlx::query_as(
            "SELECT DISTINCT brandname, brandname_ruby FROM collection_element_details",
        )
        .fetch_all(&*pool)
        .await?)
    }

    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = match is_nukige {
            true => sqlx::query_as(
                "SELECT collection_element_id from collection_element_details where is_nukige != 0",
            )
            .fetch_all(&*pool)
            .await?,
            false => sqlx::query_as(
                "SELECT collection_element_id from collection_element_details where is_nukige = 0",
            )
            .fetch_all(&*pool)
            .await?,
        };
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }
    async fn get_element_ids_by_install_at_not_null(&self) -> Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> =
            sqlx::query_as("SELECT id from collection_elements where install_at is not null")
                .fetch_all(&*pool)
                .await?;

        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }
    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let mut builder = sqlx::query_builder::QueryBuilder::new(
            "SELECT collection_element_id from collection_element_details where brandname IN (",
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
    ) -> Result<Vec<Id<CollectionElement>>> {
        let pool = self.pool.0.clone();
        let ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT collection_element_id FROM collection_element_details WHERE DATE(sellday) BETWEEN ? AND ?;",
        )
        .bind(since)
        .bind(until)
        .fetch_all(&*pool)
        .await?;
        Ok(ids.into_iter().map(|v| Id::new(v.0)).collect())
    }
    async fn update_element_last_play_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> Result<()> {
        let pool = self.pool.0.clone();
        query("update collection_elements set last_play_at = ? where id = ?")
            .bind(last_play_at.naive_utc()) // TODO: naive_utc いるか確認
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn update_element_like_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        like_at: Option<DateTime<Local>>,
    ) -> Result<()> {
        let pool = self.pool.0.clone();
        query("update collection_elements set like_at = ? where id = ?")
            .bind(like_at.and_then(|v| Some(v.naive_utc()))) // TODO: naive_utc いるか確認
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn delete_element_by_id(&self, id: &Id<CollectionElement>) -> Result<()> {
        let pool = self.pool.0.clone();
        query("delete collection_elements where id = ?")
            .bind(id.value)
            .execute(&*pool)
            .await?;
        Ok(())
    }
}
