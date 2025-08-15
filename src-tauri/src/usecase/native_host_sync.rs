use std::sync::Arc;
use derive_new::new;
use crate::domain::repository::collection::CollectionRepository;
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;

#[derive(Clone, Debug)]
pub struct DmmSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
pub struct DlsiteSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
pub struct EgsInfo {
    pub erogamescape_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
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
        for DmmSyncGameParam { store_id, category, subcategory, egs } in games {
            // 既存 (store_id, category, subcategory) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dmm_mapping(&store_id, &category, &subcategory)
                .await?;
            if exists.is_some() {
                continue;
            }
            let id = match &egs {
                Some(egs) => {
                    // EGS 情報がある場合は EGS マップも作成し、詳細も upsert
                    // 既存 EGS→collection の逆引きがあればそれを使う
                    if let Some(cid) = self
                        .repositories
                        .collection_repository()
                        .get_collection_id_by_erogamescape_id(egs.erogamescape_id)
                        .await? {
                        self.repositories
                            .collection_repository()
                            .upsert_dmm_mapping(&cid, &store_id, &category, &subcategory)
                            .await?;
                        // 詳細 upsert
                        let info = crate::domain::collection::NewCollectionElementInfo::new(
                            cid.clone(),
                            egs.gamename.clone(),
                            egs.gamename_ruby.clone(),
                            egs.brandname.clone(),
                            egs.brandname_ruby.clone(),
                            egs.sellday.clone(),
                            egs.is_nukige,
                        );
                        self.repositories.collection_repository().upsert_collection_element_info(&info).await?;
                        cid
                    } else {
                        let cid = self
                            .repositories
                            .collection_repository()
                            .allocate_new_collection_element_id()
                            .await?;
                        self.repositories
                            .collection_repository()
                            .upsert_dmm_mapping(&cid, &store_id, &category, &subcategory)
                            .await?;
                        self.repositories
                            .collection_repository()
                            .upsert_erogamescape_map(&cid, egs.erogamescape_id)
                            .await?;
                        let info = crate::domain::collection::NewCollectionElementInfo::new(
                            cid.clone(),
                            egs.gamename.clone(),
                            egs.gamename_ruby.clone(),
                            egs.brandname.clone(),
                            egs.brandname_ruby.clone(),
                            egs.sellday.clone(),
                            egs.is_nukige,
                        );
                        self.repositories.collection_repository().upsert_collection_element_info(&info).await?;
                        cid
                    }
                }
                None => {
                    // 従来通り EGS 不明としてマッピングのみ
                    let cid = self
                        .repositories
                        .collection_repository()
                        .allocate_new_collection_element_id()
                        .await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dmm_mapping(&cid, &store_id, &category, &subcategory)
                        .await?;
                    cid
                }
            };
            success += 1;
        }
        Ok(success)
    }

    pub async fn sync_dlsite_games(
        &self,
        games: Vec<DlsiteSyncGameParam>,
    ) -> anyhow::Result<u32> {
        let mut success: u32 = 0;
        for DlsiteSyncGameParam { store_id, category, egs } in games {
            // 既存 (store_id, category) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dlsite_mapping(&store_id, &category)
                .await?;
            if exists.is_some() {
                continue;
            }
            let id = match &egs {
                Some(egs) => {
                    if let Some(cid) = self
                        .repositories
                        .collection_repository()
                        .get_collection_id_by_erogamescape_id(egs.erogamescape_id)
                        .await? {
                        self.repositories
                            .collection_repository()
                            .upsert_dlsite_mapping(&cid, &store_id, &category)
                            .await?;
                        let info = crate::domain::collection::NewCollectionElementInfo::new(
                            cid.clone(),
                            egs.gamename.clone(),
                            egs.gamename_ruby.clone(),
                            egs.brandname.clone(),
                            egs.brandname_ruby.clone(),
                            egs.sellday.clone(),
                            egs.is_nukige,
                        );
                        self.repositories.collection_repository().upsert_collection_element_info(&info).await?;
                        cid
                    } else {
                        let cid = self
                            .repositories
                            .collection_repository()
                            .allocate_new_collection_element_id()
                            .await?;
                        self.repositories
                            .collection_repository()
                            .upsert_dlsite_mapping(&cid, &store_id, &category)
                            .await?;
                        self.repositories
                            .collection_repository()
                            .upsert_erogamescape_map(&cid, egs.erogamescape_id)
                            .await?;
                        let info = crate::domain::collection::NewCollectionElementInfo::new(
                            cid.clone(),
                            egs.gamename.clone(),
                            egs.gamename_ruby.clone(),
                            egs.brandname.clone(),
                            egs.brandname_ruby.clone(),
                            egs.sellday.clone(),
                            egs.is_nukige,
                        );
                        self.repositories.collection_repository().upsert_collection_element_info(&info).await?;
                        cid
                    }
                }
                None => {
                    let cid = self
                        .repositories
                        .collection_repository()
                        .allocate_new_collection_element_id()
                        .await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dlsite_mapping(&cid, &store_id, &category)
                        .await?;
                    cid
                }
            };
            success += 1;
        }
        Ok(success)
    }
}


