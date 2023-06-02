use anyhow::Result;
use async_trait::async_trait;
use sqlx::{query, query_as, QueryBuilder};

use super::{
    models::collection::{CollectionElementTable, CollectionTable},
    repository::RepositoryImpl,
};
use crate::domain::{
    collection::{Collection, CollectionElement, NewCollection, NewCollectionElement},
    repository::collection::CollectionRepository,
    Id,
};

#[async_trait]
impl CollectionRepository for RepositoryImpl<Collection> {
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
    async fn get_elements_by_id(&self, id: &Id<Collection>) -> Result<Vec<CollectionElement>> {
        let pool = self.pool.0.clone();
        Ok(query_as::<_, CollectionElementTable>(
            "select collection_elements.* from collection_element_maps inner join collection_elements on collection_elements.id = collection_element_id where collection_id = ?",
        )
        .bind(id.value)
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .filter_map(|v| v.try_into().ok())
        .collect())
    }
    async fn create(&self, new: NewCollection) -> Result<Collection> {
        if new.name.len() == 0 {
            anyhow::bail!("name is required");
        }

        let pool = self.pool.0.clone();
        let _ = query("insert into collections (name) values (?)")
            .bind(new.name)
            .execute(&*pool)
            .await?;
        Ok(query_as::<_, CollectionTable>(
            "select * from collections where id = last_insert_rowid()",
        )
        .fetch_one(&*pool)
        .await?
        .try_into()?)
    }
    async fn upsert_collection_element(&self, new: &NewCollectionElement) -> Result<()> {
        let pool = self.pool.0.clone();
        let _ = query("insert into collection_elements (id, gamename, path) values (?, ?, ?) ON CONFLICT(id) DO UPDATE SET gamename = ?, path = ?")
            .bind(new.id.value)
            .bind(new.gamename.clone())
            .bind(new.path.clone())
            .bind(new.gamename.clone())
            .bind(new.path.clone())
            .execute(&*pool)
            .await?;
        Ok(())
    }
    async fn create_collection_elements(
        &self,
        new_elements: Vec<NewCollectionElement>,
    ) -> Result<()> {
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder =
            QueryBuilder::new("INSERT INTO collection_elements (id, gamename, path) ");
        query_builder.push_values(new_elements, |mut b, new| {
            b.push_bind(new.id.value)
                .push_bind(new.gamename)
                .push_bind(new.path);
        });

        let pool = self.pool.0.clone();
        let mut query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn add_elements_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> Result<()> {
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO collection_element_maps (collection_id, collection_element_id) ",
        );
        query_builder.push_values(collection_element_ids, |mut b, new| {
            b.push_bind(collection_id.value).push_bind(new.value);
        });

        let pool = self.pool.0.clone();
        let mut query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
    async fn remove_element_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_id: &Id<CollectionElement>,
    ) -> Result<()> {
        let pool = self.pool.0.clone();
        let _ = query(
            "delete collection_element_maps where collection_id = ? AND collection_element_id = ?",
        )
        .bind(collection_id.value)
        .bind(collection_element_id.value)
        .execute(&*pool)
        .await?;
        Ok(())
    }
}
