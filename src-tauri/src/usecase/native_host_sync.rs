//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use std::sync::Arc;
use derive_new::new;
use crate::domain::repository::collection::CollectionRepository;
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;

#[derive(Clone, Debug)]
/// DMM 由来のゲーム同期パラメータ。キーは `(store_id, category, subcategory)`。
/// - `egs` が `Some` の場合、EGS 情報をコレクションへ反映し、EGS マップも作成/更新する。
/// - `egs` が `None` の場合、与えられた `gamename` を用いて要素IDを採番し、マッピングのみ作成する。
pub struct DmmSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub gamename: String,
    pub egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
/// DLsite 由来のゲーム同期パラメータ。キーは `(store_id, category)`。
/// 取り扱いは DMM と同様で、`egs` の有無に応じてコレクション側の更新内容が変わる。
pub struct DlsiteSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub gamename: String,
    pub egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
/// ErogameScape(EGS) 由来のメタ情報。
/// コレクション要素の名称・詳細情報に反映され、`erogamescape_id` は EGS マップのキーとなる。
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
/// ストア情報をコレクションへ同期するユースケース。
/// 内部で `CollectionRepository` を用いてマッピング作成・EGS 情報反映を行う。
pub struct NativeHostSyncUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> NativeHostSyncUseCase<R> {
    /// 指定 EGS に対応するコレクション要素を確実に用意する。
    /// - 既存があれば名称・詳細を上書き更新
    /// - なければ新規採番し、EGS マップ・名称・詳細を作成
    /// 戻り値: コレクション要素 ID
    async fn ensure_collection_for_egs(
        &self,
        egs: &EgsInfo,
    ) -> anyhow::Result<crate::domain::Id<crate::domain::collection::CollectionElement>> {
        let collection_element_id;
        if let Some(cid) = self
            .repositories
            .collection_repository()
            .get_collection_id_by_erogamescape_id(egs.erogamescape_id)
            .await?
        {
            collection_element_id = cid;
        } else {
            // 新規採番し、EGSマップを作成
            let cid = self
                .repositories
                .collection_repository()
                .allocate_new_collection_element_id(&egs.gamename)
                .await?;
            self.repositories
                .collection_repository()
                .upsert_erogamescape_map(&cid, egs.erogamescape_id)
                .await?;
            collection_element_id = cid;
        }

        // erogamescape 由来の詳細情報を upsert
        let info: crate::domain::collection::NewCollectionElementInfo = crate::domain::collection::NewCollectionElementInfo::new(
            collection_element_id.clone(),
            egs.gamename_ruby.clone(),
            egs.brandname.clone(),
            egs.brandname_ruby.clone(),
            egs.sellday.clone(),
            egs.is_nukige,
        );
        self
            .repositories
            .collection_repository()
            .upsert_collection_element_info(&info)
            .await?;

        Ok(collection_element_id)
    }

    /// EGS 不明用の要素を採番して作成する。
    /// - 与えられた `gamename` をそのまま `collection_elements` に登録する
    /// 戻り値: コレクション要素 ID
    async fn create_collection_without_egs(
        &self,
        gamename: &str,
    ) -> anyhow::Result<crate::domain::Id<crate::domain::collection::CollectionElement>> {
        self
            .repositories
            .collection_repository()
            .allocate_new_collection_element_id(gamename)
            .await
    }

    /// DMM のゲーム情報を同期する。
    /// - 既存チェック: `(store_id, category, subcategory)` が存在すればスキップ（冪等）
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
    /// 戻り値: 新規に作成/更新した件数
    /// エラー: 最初に失敗した地点で早期終了し伝播
    pub async fn sync_dmm_games(
        &self,
        games: Vec<DmmSyncGameParam>,
    ) -> anyhow::Result<u32> {
        let mut success: u32 = 0;
        for DmmSyncGameParam { store_id, category, subcategory, gamename, egs } in games {
            // 既存 (store_id, category, subcategory) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dmm_mapping(&store_id, &category, &subcategory)
                .await?;
            if exists.is_some() {
                continue;
            }
            match egs.as_ref() {
                Some(egs) => {
                    let cid = self.ensure_collection_for_egs(egs).await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dmm_mapping(&cid, &store_id, &category, &subcategory)
                        .await?;
                }
                None => {
                    let cid = self.create_collection_without_egs(&gamename).await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dmm_mapping(&cid, &store_id, &category, &subcategory)
                        .await?;
                }
            }
            success += 1;
        }
        Ok(success)
    }

    /// DLsite のゲーム情報を同期する。
    /// - 既存チェック: `(store_id, category)` が存在すればスキップ（冪等）
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DLsite マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DLsite マッピングのみ upsert
    /// 戻り値: 新規に作成/更新した件数
    /// エラー: 最初に失敗した地点で早期終了し伝播
    pub async fn sync_dlsite_games(
        &self,
        games: Vec<DlsiteSyncGameParam>,
    ) -> anyhow::Result<u32> {
        let mut success: u32 = 0;
        for DlsiteSyncGameParam { store_id, category, gamename, egs } in games {
            // 既存 (store_id, category) がある場合はスキップ
            let exists = self
                .repositories
                .collection_repository()
                .get_collection_id_by_dlsite_mapping(&store_id, &category)
                .await?;
            if exists.is_some() {
                continue;
            }
            match egs.as_ref() {
                Some(egs) => {
                    let cid = self.ensure_collection_for_egs(egs).await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dlsite_mapping(&cid, &store_id, &category)
                        .await?;
                }
                None => {
                    let cid = self.create_collection_without_egs(&gamename).await?;
                    self.repositories
                        .collection_repository()
                        .upsert_dlsite_mapping(&cid, &store_id, &category)
                        .await?;
                }
            }
            success += 1;
        }
        Ok(success)
    }
}


