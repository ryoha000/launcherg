use std::sync::Arc;
use derive_new::new;
use crate::domain::repository::collection::CollectionRepository;
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;

#[derive(Clone, Debug)]
pub struct DmmSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}

#[derive(Clone, Debug)]
pub struct DlsiteSyncGameParam {
    pub store_id: String,
    pub category: String,
}

#[derive(new)]
pub struct NativeHostSyncUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> NativeHostSyncUseCase<R> {
    pub async fn sync_dmm_games(
        &self,
        games: Vec<DmmSyncGameParam>,
    ) -> anyhow::Result<u32> {
        let mut success: u32 = 0;
        for DmmSyncGameParam { store_id, category, subcategory } in games {
            // EGS不明はそのまま保存（collection_element_erogamescape_mapは作らない）
            // 既存 (store_id, category, subcategory) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dmm_mapping(&store_id, &category, &subcategory)
                .await?;
            if exists.is_some() {
                continue;
            }
            let id = self
                .repositories
                .collection_repository()
                .allocate_new_collection_element_id()
                .await?;
            self.repositories
                .collection_repository()
                .upsert_dmm_mapping(&id, &store_id, &category, &subcategory)
                .await?;
            success += 1;
        }
        Ok(success)
    }

    pub async fn sync_dlsite_games(
        &self,
        games: Vec<DlsiteSyncGameParam>,
    ) -> anyhow::Result<u32> {
        let mut success: u32 = 0;
        for DlsiteSyncGameParam { store_id, category } in games {
            // 既存 (store_id, category) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dlsite_mapping(&store_id, &category)
                .await?;
            if exists.is_some() {
                continue;
            }
            let id = self
                .repositories
                .collection_repository()
                .allocate_new_collection_element_id()
                .await?;
            self.repositories
                .collection_repository()
                .upsert_dlsite_mapping(&id, &store_id, &category)
                .await?;
            success += 1;
        }
        Ok(success)
    }
}


