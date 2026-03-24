use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use domain::repository::{
    all_game_cache::AllGameCacheRepository,
    erogamescape::ErogamescapeRepository,
    manager::RepositoryManager,
    save_image_queue::ImageSaveQueueRepository,
    work_lnk::{NewWorkLnk, WorkLnkRepository},
    work_parent_packs::WorkParentPacksRepository,
    works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::save_image_queue::{ImagePreprocess, ImageSrcType};
use domain::service::save_path_resolver::SavePathResolver;
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, RegisterWorkPath, UniqueWorkKey,
    WorkRegistrationRequest, WorkRegistrationResult, WorkRegistrationService,
};
use domain::windows::shell_link::{CreateShortcutRequest, ShellLink as _};
use domain::windows::WindowsExt;

pub struct WorkRegistrationServiceImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    resolver: Arc<dyn SavePathResolver>,
    windows: Arc<W>,
    _marker: PhantomData<R>,
}

impl<M, R, W> WorkRegistrationServiceImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>, resolver: Arc<dyn SavePathResolver>, windows: Arc<W>) -> Self {
        Self {
            manager,
            resolver,
            windows,
            _marker: PhantomData,
        }
    }
}

impl<M, R, W> WorkRegistrationService for WorkRegistrationServiceImpl<M, R, W>
where
    M: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    async fn register(
        &self,
        requests: Vec<WorkRegistrationRequest>,
    ) -> anyhow::Result<Vec<WorkRegistrationResult>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let resolver = self.resolver.clone();
        let windows = self.windows.clone();

        // 事前フェッチ（N+1回避）
        let (egs_to_work, dmm_to_work, dlsite_to_work, egs_to_agc) = {
            let mut egs_ids = Vec::new();
            let mut dmm_keys = Vec::new();
            let mut dlsite_keys = Vec::new();

            for req in requests.iter() {
                for key in req.keys.iter() {
                    match key {
                        UniqueWorkKey::ErogamescapeId(id) => {
                            egs_ids.push(*id);
                        }
                        UniqueWorkKey::Dmm {
                            store_id,
                            category,
                            subcategory,
                        } => {
                            dmm_keys.push((
                                store_id.clone(),
                                category.clone(),
                                subcategory.clone(),
                            ));
                        }
                        UniqueWorkKey::Dlsite { store_id, category } => {
                            dlsite_keys.push((store_id.clone(), category.clone()));
                        }
                    }
                }
            }

            // EGS ID の重複を除去
            egs_ids.sort();
            egs_ids.dedup();

            // バッチ取得
            let (egs_map, dmm_map, dlsite_map, agc_map) = self
                .manager
                .run(|repos| {
                    let egs_ids = egs_ids.clone();
                    let dmm_keys = dmm_keys.clone();
                    let dlsite_keys = dlsite_keys.clone();
                    Box::pin(async move {
                        // EGS → Work ID
                        let mut egs_to_work: HashMap<i32, domain::StrId<domain::works::Work>> =
                            HashMap::new();
                        if !egs_ids.is_empty() {
                            let pairs = repos
                                .work()
                                .find_work_ids_by_erogamescape_ids(&egs_ids)
                                .await?;
                            for (egs, wid) in pairs.into_iter() {
                                egs_to_work.insert(egs, wid);
                            }
                        }

                        // DMM → Work ID（subcategory を含む）
                        let mut dmm_to_work: HashMap<
                            (String, String, String),
                            domain::StrId<domain::works::Work>,
                        > = HashMap::new();
                        for key in dmm_keys.iter() {
                            if let Some(dmm) = repos
                                .dmm_work()
                                .find_by_store_key(&key.0, &key.1, &key.2)
                                .await?
                            {
                                dmm_to_work.insert(key.clone(), dmm.work_id.clone());
                            }
                        }

                        // DLSITE → Work ID
                        let mut dlsite_to_work: HashMap<
                            (String, String),
                            domain::StrId<domain::works::Work>,
                        > = HashMap::new();
                        for key in dlsite_keys.iter() {
                            if let Some(dlsite) = repos
                                .dlsite_work()
                                .find_by_store_key(&key.0, &key.1)
                                .await?
                            {
                                dlsite_to_work.insert(key.clone(), dlsite.work_id.clone());
                            }
                        }

                        // EGS → AllGameCache（サムネ用）
                        let mut egs_to_agc: HashMap<
                            i32,
                            domain::all_game_cache::AllGameCacheOneWithThumbnailUrl,
                        > = HashMap::new();
                        if !egs_ids.is_empty() {
                            let list = repos.all_game_cache().get_by_ids(egs_ids).await?;
                            for gc in list.into_iter() {
                                egs_to_agc.insert(gc.id, gc);
                            }
                        }

                        Ok::<
                            (
                                HashMap<i32, domain::StrId<domain::works::Work>>,
                                HashMap<
                                    (String, String, String),
                                    domain::StrId<domain::works::Work>,
                                >,
                                HashMap<(String, String), domain::StrId<domain::works::Work>>,
                                HashMap<
                                    i32,
                                    domain::all_game_cache::AllGameCacheOneWithThumbnailUrl,
                                >,
                            ),
                            anyhow::Error,
                        >((
                            egs_to_work,
                            dmm_to_work,
                            dlsite_to_work,
                            egs_to_agc,
                        ))
                    })
                })
                .await?;

            (egs_map, dmm_map, dlsite_map, agc_map)
        };

        // 単一トランザクションで処理
        let results = self
            .manager
            .run_in_transaction(move |repos| {
                let resolver = resolver.clone();
                let windows = windows.clone();
                let requests = requests.clone();
                let egs_to_work = egs_to_work.clone();
                let dmm_to_work = dmm_to_work.clone();
                let dlsite_to_work = dlsite_to_work.clone();
                let egs_to_agc = egs_to_agc.clone();
                Box::pin(async move {
                    let mut results = Vec::new();
                    let mut exe_reqs: Vec<CreateShortcutRequest> = Vec::new();
                    let mut to_insert_lnk: Vec<(domain::StrId<domain::works::Work>, String)> =
                        Vec::new();

                    for req in requests.iter() {
                        // 既存 Work ID を取得（keys の順番で最初にヒットしたものを採用）
                        let mut existing_work_id = None;
                        for key in req.keys.iter() {
                            let found = match key {
                                UniqueWorkKey::ErogamescapeId(id) => egs_to_work.get(id).cloned(),
                                UniqueWorkKey::Dmm {
                                    store_id,
                                    category,
                                    subcategory,
                                } => dmm_to_work
                                    .get(&(store_id.clone(), category.clone(), subcategory.clone()))
                                    .cloned(),
                                UniqueWorkKey::Dlsite { store_id, category } => dlsite_to_work
                                    .get(&(store_id.clone(), category.clone()))
                                    .cloned(),
                            };
                            if found.is_some() {
                                existing_work_id = found;
                                break;
                            }
                        }

                        // 新規 Work かどうかを判定（work_id 計算前に）
                        let is_new_work = existing_work_id.is_none();

                        // Work が無ければ作成
                        let work_id = match existing_work_id {
                            Some(id) => id,
                            None => {
                                repos
                                    .work()
                                    .upsert(&domain::works::NewWork::new(req.insert.title.clone()))
                                    .await?
                            }
                        };

                        // 全 keys を順に処理してマッピングを挿入（非破壊）
                        let mut resolved_keys = Vec::new();
                        for key in req.keys.iter() {
                            match key {
                                UniqueWorkKey::ErogamescapeId(id) => {
                                    // EGS は常時追加（他Workに影響なし）
                                    repos
                                        .work()
                                        .upsert_erogamescape_map(work_id.clone(), *id)
                                        .await?;
                                    resolved_keys.push(key.clone());
                                }
                                UniqueWorkKey::Dmm {
                                    store_id,
                                    category,
                                    subcategory,
                                } => {
                                    // DMM: 事前に find_by_store_id で他Work割当をチェック
                                    let existing = dmm_to_work
                                        .get(&(
                                            store_id.clone(),
                                            category.clone(),
                                            subcategory.clone(),
                                        ))
                                        .cloned();
                                    if let Some(existing_dmm) = existing {
                                        // 既に他Workに割当済みならスキップ
                                        if existing_dmm != work_id {
                                            continue;
                                        }
                                    }
                                    // 未割当 or 同一Workなら upsert
                                    let _ = repos
                                        .dmm_work()
                                        .upsert(&domain::works::NewDmmWork::new(
                                            store_id.clone(),
                                            category.clone(),
                                            subcategory.clone(),
                                            work_id.clone(),
                                        ))
                                        .await?;
                                    resolved_keys.push(key.clone());
                                }
                                UniqueWorkKey::Dlsite { store_id, category } => {
                                    // DLsite: 事前に find_by_store_id で他Work割当をチェック
                                    let existing = dlsite_to_work
                                        .get(&(store_id.clone(), category.clone()))
                                        .cloned();
                                    if let Some(existing_dlsite) = existing {
                                        // 既に他Workに割当済みならスキップ
                                        if existing_dlsite != work_id {
                                            continue;
                                        }
                                    }
                                    // 未割当 or 同一Workなら upsert
                                    let _ = repos
                                        .dlsite_work()
                                        .upsert(&domain::works::NewDlsiteWork::new(
                                            store_id.clone(),
                                            category.clone(),
                                            work_id.clone(),
                                        ))
                                        .await?;
                                    resolved_keys.push(key.clone());
                                }
                            }
                        }

                        // EGS 情報の upsert
                        if let Some(ref egs_info) = req.insert.egs_info {
                            repos.erogamescape().upsert_information(egs_info).await?;
                        }

                        // パス登録（LNK/EXE）
                        if let Some(ref path) = req.insert.path {
                            let src_path = match path {
                                RegisterWorkPath::Lnk { lnk_path } => {
                                    to_insert_lnk.push((work_id.clone(), lnk_path.clone()));
                                    lnk_path.clone()
                                }
                                RegisterWorkPath::Exe { exe_path } => {
                                    let dst = resolver.lnk_new_path(&work_id.value);
                                    if let Some(parent) = Path::new(&dst).parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    let working_dir = Path::new(exe_path)
                                        .parent()
                                        .map(|p| p.display().to_string());
                                    exe_reqs.push(CreateShortcutRequest {
                                        target_path: exe_path.clone(),
                                        dest_lnk_path: dst.clone(),
                                        working_dir,
                                        arguments: None,
                                        icon_path: None,
                                    });
                                    to_insert_lnk.push((work_id.clone(), dst));
                                    exe_path.clone()
                                }
                            };

                            // install_at と original_path を記録
                            if let Ok(meta) = std::fs::metadata(&src_path) {
                                let created = meta.created().ok();
                                let modified = meta.modified().ok();
                                if let Some(best_st) = match (created, modified) {
                                    (Some(c), Some(m)) => Some(if m > c { m } else { c }),
                                    (Some(c), None) => Some(c),
                                    (None, Some(m)) => Some(m),
                                    _ => None,
                                } {
                                    // SystemTime を chrono::DateTime<Local> に変換
                                    let best_dt_local =
                                        chrono::DateTime::<chrono::Utc>::from(best_st)
                                            .with_timezone(&chrono::Local);
                                    if let Err(e) = repos
                                        .work()
                                        .update_install_by_work_id(
                                            work_id.clone(),
                                            best_dt_local,
                                            src_path.clone(),
                                        )
                                        .await
                                    {
                                        log::warn!(
                                            "Failed to update install_at for work_id={}: {}",
                                            work_id.value,
                                            e
                                        );
                                    }
                                }
                            } else {
                                log::warn!(
                                    "Failed to get metadata for path: {}",
                                    src_path
                                );
                            }
                        }

                        // アイコン処理
                        if let Some(ref icon_apply) = req.insert.icon {
                            let icon_dst = resolver.icon_png_path(&work_id.value);
                            let icon_exists = Path::new(&icon_dst).exists();

                            if should_apply_image(
                                icon_apply.strategy.clone(),
                                is_new_work,
                                icon_exists,
                            ) {
                                if let Some((src_path, src_type)) =
                                    resolve_image_src(req, icon_apply, &egs_to_agc)
                                {
                                    repos
                                        .image_queue()
                                        .enqueue(
                                            &src_path,
                                            src_type,
                                            &icon_dst,
                                            ImagePreprocess::ResizeAndCropSquare256,
                                        )
                                        .await?;
                                }
                            }
                        }

                        // サムネイル処理
                        if let Some(ref thumb_apply) = req.insert.thumbnail {
                            let thumb_dst = resolver.thumbnail_png_path(&work_id.value);
                            let thumb_exists = Path::new(&thumb_dst).exists();

                            if should_apply_image(
                                thumb_apply.strategy.clone(),
                                is_new_work,
                                thumb_exists,
                            ) {
                                if let Some((src_path, src_type)) =
                                    resolve_image_src(req, thumb_apply, &egs_to_agc)
                                {
                                    repos
                                        .image_queue()
                                        .enqueue(
                                            &src_path,
                                            src_type,
                                            &thumb_dst,
                                            ImagePreprocess::ResizeForWidth400,
                                        )
                                        .await?;
                                }
                            }
                        }

                        // 親パック関連付け
                        if let Some(parent_pack_key) = req.insert.parent_pack_dmm_key.clone() {
                            repos
                                .work_parent_packs()
                                .add(work_id.clone(), parent_pack_key)
                                .await?;
                        }

                        results.push(WorkRegistrationResult {
                            resolved_keys,
                            work_id: work_id.clone(),
                            is_new_work,
                        });
                    }

                    // EXE の .lnk を一括作成
                    if !exe_reqs.is_empty() {
                        windows.shell_link().create_bulk(exe_reqs)?;
                    }

                    // work_lnk に登録
                    for (work_id, lnk_path) in to_insert_lnk.into_iter() {
                        repos
                            .work_lnk()
                            .insert(&NewWorkLnk { work_id, lnk_path })
                            .await?;
                    }

                    Ok::<Vec<WorkRegistrationResult>, anyhow::Error>(results)
                })
            })
            .await?;

        Ok(results)
    }
}

/// 画像適用の戦略に基づいて、適用すべきかどうかを判定する
fn should_apply_image(strategy: ImageStrategy, is_new_work: bool, dst_exists: bool) -> bool {
    match strategy {
        ImageStrategy::Always => true,
        ImageStrategy::OnlyIfNew => is_new_work,
        ImageStrategy::OnlyIfMissing => !dst_exists,
        ImageStrategy::Never => false,
    }
}

/// 画像ソースを解決し、ソースパスとImageSrcTypeを返す
fn resolve_image_src(
    req: &WorkRegistrationRequest,
    apply: &ImageApply,
    egs_to_agc: &HashMap<i32, domain::all_game_cache::AllGameCacheOneWithThumbnailUrl>,
) -> Option<(String, ImageSrcType)> {
    match &apply.source {
        ImageSource::FromUrl(url) => Some((url.clone(), ImageSrcType::Url)),
        ImageSource::FromEgs => {
            // EGS から取得（keys 内の EGS キーを探索）
            let mut found_url = None;
            for key in req.keys.iter() {
                if let UniqueWorkKey::ErogamescapeId(id) = key {
                    if let Some(gc) = egs_to_agc.get(id) {
                        if !gc.thumbnail_url.is_empty() {
                            found_url = Some(gc.thumbnail_url.clone());
                            break;
                        }
                    }
                }
            }
            found_url.map(|url| (url, ImageSrcType::Url))
        }
        ImageSource::FromPath(path) => {
            let (src_path, src_type) = match path {
                RegisterWorkPath::Lnk { lnk_path } => (lnk_path.clone(), ImageSrcType::Shortcut),
                RegisterWorkPath::Exe { exe_path } => (exe_path.clone(), ImageSrcType::Exe),
            };
            Some((src_path, src_type))
        }
    }
}

#[cfg(test)]
mod tests;
