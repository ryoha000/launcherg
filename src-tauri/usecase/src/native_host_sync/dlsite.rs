use super::*;
use crate::native_host_sync::store::{PlanDecisionGeneric, StoreOps};
use domain::repository::save_image_queue::ImageSaveQueueRepository as _;
use domain::save_image_queue::{ImagePreprocess, ImageSrcType};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DlsiteKey {
    store_id: String,
    category: String,
}

// 旧スナップショット/適用関連は共通パイプラインへ移行済み

impl<M, R> NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn sync_dlsite_games(&self, games: Vec<DlsiteSyncGameParam>) -> anyhow::Result<u32> {
        // 構成子
        fn key_from_param(p: &DlsiteSyncGameParam) -> DlsiteKey {
            DlsiteKey {
                store_id: p.store_id.clone(),
                category: p.category.clone(),
            }
        }
        fn gamename(p: &DlsiteSyncGameParam) -> &str {
            &p.gamename
        }
        fn egs(p: &DlsiteSyncGameParam) -> Option<&EgsInfo> {
            p.egs.as_ref()
        }
        fn parent_pack_work_id(_: &DlsiteSyncGameParam) -> Option<domain::Id<domain::works::Work>> {
            None
        }

        fn find_work_id_by_key<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            repos: &'a Rx,
            k: &'a DlsiteKey,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<Option<domain::Id<domain::works::Work>>>>
        {
            Box::pin(async move {
                let mut repo = repos.dlsite_work();
                Ok(repo
                    .find_by_store_key(&k.store_id, &k.category)
                    .await?
                    .map(|w| w.work_id))
            })
        }
        fn upsert_store_mapping<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            repos: &'a Rx,
            k: &'a DlsiteKey,
            work_id: domain::Id<domain::works::Work>,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
            Box::pin(async move {
                let mut repo = repos.dlsite_work();
                let _ = repo
                    .upsert(&domain::works::NewDlsiteWork::new(
                        k.store_id.clone(),
                        k.category.clone(),
                        work_id,
                    ))
                    .await?;
                Ok(())
            })
        }
        fn link_parent_pack_if_needed<'a, Rx: RepositoriesExt + Send + Sync + 'static>(
            _: &'a Rx,
            _: domain::Id<domain::works::Work>,
            _: Option<domain::Id<domain::works::Work>>,
        ) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
            Box::pin(async move { Ok(()) })
        }

        let ops: StoreOps<DlsiteSyncGameParam, DlsiteKey, R> = StoreOps {
            key_from_param,
            gamename,
            egs,
            parent_pack_work_id,
            find_work_id_by_key: find_work_id_by_key::<R>,
            upsert_store_mapping: upsert_store_mapping::<R>,
            link_parent_pack_if_needed: link_parent_pack_if_needed::<R>,
        };

        // key でユニーク化し、同時に key→image_url のマップを構築
        let mut seen: HashSet<DlsiteKey> = HashSet::new();
        let mut unique_games: Vec<DlsiteSyncGameParam> = Vec::with_capacity(games.len());
        let mut image_url_by_key: HashMap<DlsiteKey, String> = HashMap::new();
        for g in games.into_iter() {
            let k = DlsiteKey {
                store_id: g.store_id.clone(),
                category: g.category.clone(),
            };
            if seen.insert(k.clone()) {
                unique_games.push(g.clone());
                image_url_by_key.insert(k, g.image_url.clone());
            }
        }

        let snapshot = self.build_batch_snapshot(&unique_games, &ops).await?;
        let mut plans: Vec<PlanDecisionGeneric<DlsiteKey>> = Vec::with_capacity(unique_games.len());
        for param in unique_games.iter().cloned() {
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
                                                let icon_dst = resolver.icon_png_path(work_id.value);
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
                                                let thumb_dst = resolver.thumbnail_png_path(work_id.value);
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
