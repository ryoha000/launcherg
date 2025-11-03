use super::*;
use domain::repository::work_omit::WorkOmitRepository;
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, UniqueWorkKey, WorkInsert,
};
use std::collections::HashSet;

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

impl<M, R, RS> NativeHostSyncUseCase<M, R, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    RS: domain::service::work_registration::WorkRegistrationService + Send + Sync + 'static,
{
    /// DMM のゲーム情報を同期する。
    /// - 既存チェック: `(store_id, category, subcategory)` が存在すればスキップ（冪等）
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
    /// 戻り値: 新規に作成/更新した件数
    /// エラー: 最初に失敗した地点で早期終了し伝播
    pub async fn sync_dmm_games(&self, games: Vec<DmmSyncGameParam>) -> anyhow::Result<u32> {
        // key でユニーク化
        let mut seen: HashSet<DmmKey> = HashSet::new();
        let mut unique_games: Vec<DmmSyncGameParam> = Vec::with_capacity(games.len());
        for g in games.into_iter() {
            let k = DmmKey::from_param(&g);
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
            DmmKey,
            Option<domain::StrId<domain::works::Work>>,
        > = std::collections::HashMap::new();
        for game in unique_games.iter() {
            let key = DmmKey::from_param(game);
            let work_id = self
                .manager
                .run(|repos| {
                    let key = key.clone();
                    Box::pin(async move {
                        let mut repo = repos.dmm_work();
                        Ok(repo
                            .find_by_store_key(&key.store_id, &key.category, &key.subcategory)
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
            let key = DmmKey::from_param(&game);
            let work_id = work_id_by_key.get(&key).cloned().flatten();

            // 除外対象ならスキップ
            if let Some(ref wid) = work_id {
                if omitted_work_ids.contains(wid) {
                    continue;
                }
            }

            // keys の構築（EGS があれば優先）
            let mut keys = Vec::new();
            keys.push(UniqueWorkKey::Dmm {
                store_id: game.store_id.clone(),
                category: game.category.clone(),
                subcategory: game.subcategory.clone(),
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

            // 親パック Work ID
            let parent_pack_work_id = game
                .parent_pack_work_id
                .as_ref()
                .map(|s| domain::StrId::new(s.clone()));

            requests.push(
                domain::service::work_registration::WorkRegistrationRequest {
                    keys,
                    insert: WorkInsert {
                        title: game.gamename,
                        path: None,
                        egs_info,
                        icon,
                        thumbnail,
                        parent_pack_work_id,
                    },
                },
            );
        }

        // WorkRegistrationService で一括登録
        let results = self.registrar.register(requests).await?;
        Ok(results.len() as u32)
    }
}
