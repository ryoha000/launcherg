use super::*;
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, UniqueWorkKey, WorkInsert,
};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DlsiteKey {
    store_id: String,
    category: String,
}

// 旧スナップショット/適用関連は共通パイプラインへ移行済み

impl<M, R, RS> NativeHostSyncUseCase<M, R, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    RS: domain::service::work_registration::WorkRegistrationService + Send + Sync + 'static,
{
    pub async fn sync_dlsite_games(
        &self,
        games: Vec<DlsiteSyncGameParam>,
    ) -> anyhow::Result<SyncGamesSummary> {
        // key でユニーク化
        let mut seen: HashSet<DlsiteKey> = HashSet::new();
        let mut unique_games: Vec<DlsiteSyncGameParam> = Vec::with_capacity(games.len());
        for g in games.into_iter() {
            let k = DlsiteKey {
                store_id: g.store_id.clone(),
                category: g.category.clone(),
            };
            if seen.insert(k) {
                unique_games.push(g);
            }
        }

        let mut requests: Vec<domain::service::work_registration::WorkRegistrationRequest> =
            Vec::new();
        for game in unique_games.into_iter() {
            // keys の構築（EGS があれば優先）
            let mut keys = Vec::new();
            keys.push(UniqueWorkKey::Dlsite {
                store_id: game.store_id.clone(),
                category: game.category.clone(),
            });
            if let Some(ref egs_info) = game.egs {
                keys.push(UniqueWorkKey::ErogamescapeId(egs_info.erogamescape_id));
            }

            // EGS 情報の構築
            let egs_info = game.egs.as_ref().map(|e| {
                domain::erogamescape::NewErogamescapeInformation::new(
                    e.erogamescape_id,
                    if e.gamename_ruby.is_empty() {
                        game.gamename.clone()
                    } else {
                        e.gamename_ruby.clone()
                    },
                    e.brandname.clone(),
                    e.brandname_ruby.clone(),
                    e.sellday.clone(),
                    e.is_nukige,
                )
            });

            // 画像処理の設定
            let icon = if !game.image_url.is_empty() {
                Some(ImageApply {
                    strategy: ImageStrategy::OnlyIfNew,
                    source: ImageSource::FromUrl(game.image_url.clone()),
                })
            } else {
                None
            };
            let thumbnail = if !game.image_url.is_empty() {
                Some(ImageApply {
                    strategy: ImageStrategy::OnlyIfNew,
                    source: ImageSource::FromUrl(normalize_thumbnail_url(&game.image_url)),
                })
            } else {
                None
            };

            requests.push(
                domain::service::work_registration::WorkRegistrationRequest {
                    keys,
                    insert: WorkInsert {
                        title: game.gamename,
                        path: None,
                        egs_info,
                        icon,
                        thumbnail,
                        parent_pack_dmm_key: None,
                    },
                },
            );
        }

        // WorkRegistrationService で一括登録
        let results = self.registrar.register(requests).await?;
        Ok(SyncGamesSummary {
            success_count: results.len() as u32,
            new_count: results.iter().filter(|result| result.is_new_work).count() as u32,
        })
    }
}
