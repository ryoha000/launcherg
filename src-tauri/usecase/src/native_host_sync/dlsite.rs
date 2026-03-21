use super::*;
use domain::repository::work_omit::WorkOmitRepository;
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
    pub async fn sync_dlsite_games(&self, games: Vec<DlsiteSyncGameParam>) -> anyhow::Result<u32> {
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

        // 除外対象 Work ID を事前に取得（別トランザクション）
        let omitted_work_ids: HashSet<domain::StrId<domain::works::Work>> = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work_omit();
                    let list = repo.list().await?;
                    Ok(list.into_iter().map(|o| o.work_id).collect())
                })
            })
            .await?;

        // 既存 Work ID を取得（ストアキーごと）
        let mut work_id_by_key: std::collections::HashMap<
            DlsiteKey,
            Option<domain::StrId<domain::works::Work>>,
        > = std::collections::HashMap::new();
        for game in unique_games.iter() {
            let key = DlsiteKey {
                store_id: game.store_id.clone(),
                category: game.category.clone(),
            };
            let work_id = self
                .manager
                .run(|repos| {
                    let key = key.clone();
                    Box::pin(async move {
                        let mut repo = repos.dlsite_work();
                        Ok(repo
                            .find_by_store_key(&key.store_id, &key.category)
                            .await?
                            .map(|w| w.work_id))
                    })
                })
                .await?;
            work_id_by_key.insert(key, work_id);
        }

        // omit 対象をフィルタ
        let mut requests: Vec<domain::service::work_registration::WorkRegistrationRequest> =
            Vec::new();
        for game in unique_games.into_iter() {
            let key = DlsiteKey {
                store_id: game.store_id.clone(),
                category: game.category.clone(),
            };
            let work_id = work_id_by_key.get(&key).cloned().flatten();

            // 除外対象ならスキップ
            if let Some(ref wid) = work_id {
                if omitted_work_ids.contains(wid) {
                    continue;
                }
            }

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
                        parent_pack_work_id: None,
                    },
                },
            );
        }

        // WorkRegistrationService で一括登録
        let results = self.registrar.register(requests).await?;
        Ok(results.len() as u32)
    }
}
