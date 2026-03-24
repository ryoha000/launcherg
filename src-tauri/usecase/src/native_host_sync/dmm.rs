use super::*;
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

impl DmmKey {
    fn from_game(game: &DmmSyncGameParam) -> Self {
        Self {
            store_id: game.store_id.clone(),
            category: game.category.clone(),
            subcategory: game.subcategory.clone(),
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
    /// - 親 pack は作品として登録せず、子作品の登録時に親 pack の DMM キーをそのまま保存する
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
    pub async fn sync_dmm_games(
        &self,
        games: Vec<DmmSyncGameParam>,
    ) -> anyhow::Result<SyncGamesSummary> {
        log::info!("sync_dmm_games start: {}", games.len());

        let mut seen = HashSet::<DmmKey>::new();
        let mut unique_games = Vec::with_capacity(games.len());
        for game in games {
            if seen.insert(DmmKey::from_game(&game)) {
                unique_games.push(game);
            }
        }

        log::info!("sync_dmm_games unique count: {}", unique_games.len());

        let requests = unique_games
            .iter()
            .map(|game| {
                let mut keys = vec![UniqueWorkKey::Dmm {
                    store_id: game.store_id.clone(),
                    category: game.category.clone(),
                    subcategory: game.subcategory.clone(),
                }];

                if let Some(egs_info) = &game.egs {
                    keys.push(UniqueWorkKey::ErogamescapeId(egs_info.erogamescape_id));
                }

                let egs_info = game.egs.as_ref().map(|egs| {
                    domain::erogamescape::NewErogamescapeInformation::new(
                        egs.erogamescape_id,
                        if egs.gamename_ruby.is_empty() {
                            game.gamename.clone()
                        } else {
                            egs.gamename_ruby.clone()
                        },
                        egs.brandname.clone(),
                        egs.brandname_ruby.clone(),
                        egs.sellday.clone(),
                        egs.is_nukige,
                    )
                });

                let icon = if game.image_url.is_empty() {
                    None
                } else {
                    Some(ImageApply {
                        strategy: ImageStrategy::OnlyIfNew,
                        source: ImageSource::FromUrl(game.image_url.clone()),
                    })
                };
                let thumbnail = if game.image_url.is_empty() {
                    None
                } else {
                    Some(ImageApply {
                        strategy: ImageStrategy::OnlyIfNew,
                        source: ImageSource::FromUrl(normalize_thumbnail_url(&game.image_url)),
                    })
                };

                let parent_pack_dmm_key = game.parent_pack.as_ref().map(|parent| domain::work_parent_pack::ParentPackKey {
                    store_id: parent.store_id.clone(),
                    category: parent.category.clone(),
                    subcategory: parent.subcategory.clone(),
                });

                domain::service::work_registration::WorkRegistrationRequest {
                    keys,
                    insert: WorkInsert {
                        title: game.gamename.clone(),
                        path: None,
                        egs_info,
                        icon,
                        thumbnail,
                        parent_pack_dmm_key,
                    },
                }
            })
            .collect::<Vec<_>>();

        let results = if requests.is_empty() {
            Vec::new()
        } else {
            self.registrar.register(requests).await?
        };
        Ok(SyncGamesSummary {
            success_count: results.len() as u32,
            new_count: results.iter().filter(|result| result.is_new_work).count() as u32,
        })
    }
}
