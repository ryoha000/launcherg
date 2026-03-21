use super::*;
use domain::repository::work_omit::WorkOmitRepository;
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, UniqueWorkKey, WorkInsert,
};
use std::collections::{HashMap, HashSet};

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

    fn from_pack(pack: &DmmPackKey) -> Self {
        Self {
            store_id: pack.store_id.clone(),
            category: pack.category.clone(),
            subcategory: pack.subcategory.clone(),
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
    /// - 先に pack 親を含まない作品を登録して work_id を確定し、その後 child に parent_pack_work_id を埋める
    /// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
    /// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
    pub async fn sync_dmm_games(&self, games: Vec<DmmSyncGameParam>) -> anyhow::Result<u32> {
        log::info!("sync_dmm_games start: {}", games.len());

        let mut seen = HashSet::<DmmKey>::new();
        let mut unique_games = Vec::with_capacity(games.len());
        for game in games {
            if seen.insert(DmmKey::from_game(&game)) {
                unique_games.push(game);
            }
        }

        let root_count = unique_games.iter().filter(|game| game.parent_pack.is_none()).count();
        let child_count = unique_games.len().saturating_sub(root_count);
        log::info!("sync_dmm_games split: roots={}, children={}", root_count, child_count);

        let omitted_work_ids: HashSet<domain::StrId<domain::works::Work>> = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work_omit();
                    let list = repo.list().await?;
                    Ok(list.into_iter().map(|item| item.work_id).collect())
                })
            })
            .await?;

        let mut keys_to_lookup: Vec<DmmKey> = unique_games.iter().map(DmmKey::from_game).collect();
        for game in &unique_games {
            if let Some(parent) = &game.parent_pack {
                keys_to_lookup.push(DmmKey::from_pack(parent));
            }
        }
        keys_to_lookup.sort_by(|a, b| {
            (a.store_id.as_str(), a.category.as_str(), a.subcategory.as_str())
                .cmp(&(b.store_id.as_str(), b.category.as_str(), b.subcategory.as_str()))
        });
        keys_to_lookup.dedup();

        let existing_work_ids: HashMap<DmmKey, domain::StrId<domain::works::Work>> = self
            .manager
            .run(|repos| {
                let keys = keys_to_lookup.clone();
                Box::pin(async move {
                    let mut map = HashMap::new();
                    let mut repo = repos.dmm_work();
                    for key in keys {
                        if let Some(work) = repo
                            .find_by_store_key(&key.store_id, &key.category, &key.subcategory)
                            .await?
                        {
                            map.insert(key, work.work_id);
                        }
                    }
                    Ok::<_, anyhow::Error>(map)
                })
            })
            .await?;

        let build_request = |game: &DmmSyncGameParam| {
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

            domain::service::work_registration::WorkRegistrationRequest {
                keys,
                insert: WorkInsert {
                    title: game.gamename.clone(),
                    path: None,
                    egs_info,
                    icon,
                    thumbnail,
                    parent_pack_work_id: None,
                },
            }
        };

        let mut root_requests: Vec<(DmmKey, domain::service::work_registration::WorkRegistrationRequest)> =
            Vec::new();
        let mut child_requests: Vec<(
            DmmKey,
            DmmKey,
            domain::service::work_registration::WorkRegistrationRequest,
        )> = Vec::new();

        for game in &unique_games {
            let key = DmmKey::from_game(game);
            if let Some(work_id) = existing_work_ids.get(&key) {
                if omitted_work_ids.contains(work_id) {
                    log::debug!("skip omitted work: {}", key.store_id);
                    continue;
                }
            }

            let request = build_request(game);
            if let Some(parent_pack) = &game.parent_pack {
                child_requests.push((key, DmmKey::from_pack(parent_pack), request));
            } else {
                root_requests.push((key, request));
            }
        }

        let root_only_requests: Vec<_> = root_requests.iter().map(|(_, request)| request.clone()).collect();
        let root_results = if root_only_requests.is_empty() {
            Vec::new()
        } else {
            self.registrar.register(root_only_requests).await?
        };

        let mut resolved_work_ids = existing_work_ids;
        for ((key, _), result) in root_requests.iter().zip(root_results.iter()) {
            resolved_work_ids.insert(key.clone(), result.work_id.clone());
        }

        let mut child_only_requests = Vec::with_capacity(child_requests.len());
        for (child_key, parent_key, mut request) in child_requests {
            if let Some(parent_work_id) = resolved_work_ids.get(&parent_key).cloned() {
                request.insert.parent_pack_work_id = Some(parent_work_id);
            } else {
                log::warn!(
                    "parent pack work id not found: child={} parent={}",
                    child_key.store_id,
                    parent_key.store_id
                );
            }
            child_only_requests.push(request);
        }

        let child_results = if child_only_requests.is_empty() {
            Vec::new()
        } else {
            self.registrar.register(child_only_requests).await?
        };
        Ok((root_results.len() + child_results.len()) as u32)
    }
}
