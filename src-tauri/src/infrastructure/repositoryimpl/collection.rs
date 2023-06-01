use anyhow::Result;
use async_trait::async_trait;
use sqlx::{query, query_as};

use super::{
    models::collection::{CollectionElementTable, CollectionTable},
    repository::RepositoryImpl,
};
use crate::domain::{
    collection::{Collection, CollectionElement, CollectionID, NewCollection},
    repository::collection::CollectionRepository,
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
    async fn get_elements_by_id(&self, id: CollectionID) -> Result<Vec<CollectionElement>> {
        let pool = self.pool.0.clone();
        Ok(query_as::<_, CollectionElementTable>(
            "select * from collection_element_maps where collection_id = ?",
        )
        .bind(id)
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
}
