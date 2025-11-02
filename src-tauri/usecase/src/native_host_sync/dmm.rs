use super::store::{PlanDecisionGeneric, StoreOps};
use super::*;
use domain::repository::save_image_queue::ImageSaveQueueRepository as _;
use domain::save_image_queue::{ImagePreprocess, ImageSrcType};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DmmKey {
    store_id: String,
    category: String,
    subcategory: String,
}

// 旧スナップショット/適用関連は共通パイプラインへ移行済み

impl DmmKey {
    fn from_param(p: &DmmSyncGameParam) -> DmmKey {
        DmmKey {
            store_id: p.store_id.clone(),
            category: p.category.clone(),
            subcategory: p.subcategory.clone(),
        }
    }
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    /// DMM のゲーム情報を同期する。
    /// - 既存チェック: `(store_id, category, subcategory)` が存在すればスキップ（冪等）
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
    /// 戻り値: 新規に作成/更新した件数
    /// エラー: 最初に失敗した地点で早期終了し伝播
    pub async fn sync_dmm_games(&self, games: Vec<DmmSyncGameParam>) -> anyhow::Result<u32> {
        // 構成子
        fn key_from_param(p: &DmmSyncGameParam) -> DmmKey {
            DmmKey::from_param(p)
        }
        fn gamename(p: &DmmSyncGameParam) -> &str {
            &p.gamename
        }
        fn egs(p: &DmmSyncGameParam) -> Option<&EgsInfo> {
            p.egs.as_ref()
        }
        fn parent_pack_work_id(p: &DmmSyncGameParam) -> Option<domain::StrId<domain::works::Work>> {
            p.parent_pack_work_id.as_ref().map(|s| domain::StrId::new(s.clone()))
        }

        fn find_work_id_by_key<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            repos: &'a Rx,
            k: &'a DmmKey,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<Option<domain::StrId<domain::works::Work>>>>
        {
            Box::pin(async move {
                let mut repo = repos.dmm_work();
                Ok(repo
                    .find_by_store_key(&k.store_id, &k.category, &k.subcategory)
                    .await?
                    .map(|w| w.work_id))
            })
        }
        fn upsert_store_mapping<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            repos: &'a Rx,
            k: &'a DmmKey,
            work_id: domain::StrId<domain::works::Work>,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
            Box::pin(async move {
                let mut repo = repos.dmm_work();
                let _ = repo
                    .upsert(&domain::works::NewDmmWork::new(
                        k.store_id.clone(),
                        k.category.clone(),
                        k.subcategory.clone(),
                        work_id,
                    ))
                    .await?;
                Ok(())
            })
        }
        fn link_parent_pack_if_needed<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            repos: &'a Rx,
            work_id: domain::StrId<domain::works::Work>,
            parent: Option<domain::StrId<domain::works::Work>>,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
            Box::pin(async move {
                if let Some(pid) = parent {
                    repos.work_parent_packs().add(work_id, pid).await?;
                }
                Ok(())
            })
        }

        let ops: StoreOps<DmmSyncGameParam, DmmKey, R> = StoreOps {
            key_from_param,
            gamename,
            egs,
            parent_pack_work_id,
            find_work_id_by_key: find_work_id_by_key::<R>,
            upsert_store_mapping: upsert_store_mapping::<R>,
            link_parent_pack_if_needed: link_parent_pack_if_needed::<R>,
        };

        // key でユニーク化し、同時に key→image_url のマップを構築
        let mut seen: HashSet<DmmKey> = HashSet::new();
        let mut unique_games: Vec<DmmSyncGameParam> = Vec::with_capacity(games.len());
        let mut image_url_by_key: HashMap<DmmKey, String> = HashMap::new();
        for g in games.into_iter() {
            let k = DmmKey::from_param(&g);
            if seen.insert(k.clone()) {
                unique_games.push(g.clone());
                image_url_by_key.insert(k, g.image_url.clone());
            }
        }

        let snapshot = self.build_batch_snapshot(&unique_games, &ops).await?;
        let mut plans: Vec<PlanDecisionGeneric<DmmKey>> = Vec::with_capacity(unique_games.len());
        for param in unique_games.into_iter() {
            plans.push(self.decide_for_game_generic(&snapshot, param, &ops).await?);
        }
        let resolver = self.resolver.clone();
        self.manager
            .run_in_transaction(move |repos| {
                let plans = plans.clone();
                let resolver = resolver.clone();
                let ops = ops;
                let image_url_by_key = image_url_by_key.clone();
                Box::pin(async move {
                    let mut new_work_count: u32 = 0;
                    for plan in plans.into_iter() {
                        match plan {
                            PlanDecisionGeneric::SkipExists | PlanDecisionGeneric::SkipOmitted => {}
                            PlanDecisionGeneric::Apply(apply) => {
                                let was_new = apply.work_id_by_key.is_none()
                                    && apply.work_id_by_erogamescape.is_none();
                                let key = apply.key.clone();
                                Self::execute_apply_with_repos_generic(
                                    &repos,
                                    apply,
                                    &ops,
                                )
                                .await?;
                                if was_new {
                                    new_work_count += 1;
                                    // 新規 Work 作成時のみ画像キューへ投入
                                    if let Some(work_id) = (ops.find_work_id_by_key)(&repos, &key).await? {
                                        if let Some(image_url) = image_url_by_key.get(&key) {
                                            if !image_url.is_empty() {
                                                let icon_dst = resolver.icon_png_path(&work_id.value);
                                                let mut repo = repos.image_queue();
                                                let _ = repo
                                                    .enqueue(
                                                        image_url,
                                                        ImageSrcType::Url,
                                                        &icon_dst,
                                                        ImagePreprocess::ResizeAndCropSquare256,
                                                    )
                                                    .await;
                                                let normalized = normalize_thumbnail_url(image_url);
                                                let thumb_dst = resolver.thumbnail_png_path(&work_id.value);
                                                let _ = repo
                                                    .enqueue(
                                                        &normalized,
                                                        ImageSrcType::Url,
                                                        &thumb_dst,
                                                        ImagePreprocess::ResizeForWidth400,
                                                    )
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(new_work_count)
                })
            })
            .await
    }
}
