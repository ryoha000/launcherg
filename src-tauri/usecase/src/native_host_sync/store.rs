use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use domain::StrId;
use domain::repository::erogamescape::ErogamescapeRepository;

use super::*;

/// ストア同期で使用する操作群。ジェネリックに定義して DMM/DLsite で共通利用。
#[derive(Clone)]
pub struct StoreOps<P, K, R>
where
    R: RepositoriesExt + Send + Sync + 'static,
{
    /// パラメータからキーを抽出
    pub key_from_param: fn(&P) -> K,
    /// パラメータからゲーム名を取得
    pub gamename: fn(&P) -> &str,
    /// パラメータから EGS 情報を取得
    pub egs: fn(&P) -> Option<&EgsInfo>,
    /// パラメータから親パック Work ID を取得
    pub parent_pack_work_id: fn(&P) -> Option<StrId<domain::works::Work>>,

    /// キーで Work ID を検索（返却: Option<WorkId>）
    pub find_work_id_by_key: for<'a> fn(
        &'a R,
        &'a K,
    ) -> Pin<
        Box<dyn Future<Output = anyhow::Result<Option<StrId<domain::works::Work>>>> + Send + 'a>,
    >,
    /// ストアマッピングを upsert（キーと Work ID の関連付け）
    pub upsert_store_mapping:
        for<'a> fn(
            &'a R,
            &'a K,
            StrId<domain::works::Work>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>,
    /// 親パック Work との関連付けが必要なら実行
    pub link_parent_pack_if_needed:
        for<'a> fn(
            &'a R,
            StrId<domain::works::Work>,
            Option<StrId<domain::works::Work>>,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>,
}


/// バッチ同期前のスナップショット。既存の Work マッピングと除外情報を保持。
#[derive(Clone, Debug)]
pub struct BatchSnapshot<K> {
    /// ストアキーごとの既存 Work ID（None = 未マッピング）
    pub work_id_by_key: HashMap<K, Option<StrId<domain::works::Work>>>,
    /// 除外対象 Work ID（同期対象外）
    pub omitted_work_ids: HashSet<StrId<domain::works::Work>>,
    /// EGS ID ごとの既存 Work ID（EGS マッピング用）
    pub egs_id_to_work_id: HashMap<i32, StrId<domain::works::Work>>,
}

/// 同期適用パラメータ。キーと関連 Work/Egs 情報を持つ。
#[derive(Clone, Debug)]
pub struct SyncApplyGeneric<K> {
    /// ストアキー
    pub key: K,
    /// ストアキー由来の既存 Work ID
    pub work_id_by_key: Option<StrId<domain::works::Work>>,
    /// EGS 由来の既存 Work ID
    pub work_id_by_erogamescape: Option<StrId<domain::works::Work>>,
    /// ゲーム名
    pub gamename: String,
    /// 親パック Work ID（DMM パックの場合）
    pub parent_pack_work_id: Option<StrId<domain::works::Work>>,
    /// EGS 情報（あれば Work に upsert）
    pub egs: Option<EgsInfo>,
}

/// 同期計画の決定結果。
#[derive(Clone, Debug)]
pub enum PlanDecisionGeneric<K> {
    /// 既に存在するためスキップ
    SkipExists,
    /// 除外対象のためスキップ
    SkipOmitted,
    /// 新規/更新適用
    Apply(SyncApplyGeneric<K>),
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    /// バッチ同期前のスナップショットを作成。
    /// - ストアキーごとの既存 Work ID を検索
    /// - 除外対象 Work を取得
    /// - EGS ID ごとの既存 Work ID を検索
    pub async fn build_batch_snapshot<P, K>(
        &self,
        games: &[P],
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<BatchSnapshot<K>>
    where
        K: Clone + std::cmp::Eq + std::hash::Hash + Send + Sync + 'static,
    {
        // ストアキー一覧
        let keys: Vec<K> = games.iter().map(|g| (ops.key_from_param)(g)).collect();

        // ストアキーごとの既存 Work ID を検索
        let mut work_id_by_key: HashMap<K, Option<StrId<domain::works::Work>>> = HashMap::new();
        let found_map = self
            .manager
            .run(|repos| {
                let keys = keys.clone();
                let ops = ops;
                Box::pin(async move {
                    let mut out = HashMap::new();
                    for k in keys.iter() {
                        let wid = (ops.find_work_id_by_key)(&repos, k).await?;
                        out.insert(k.clone(), wid);
                    }
                    Ok::<HashMap<K, Option<StrId<domain::works::Work>>>, anyhow::Error>(out)
                })
            })
            .await?;
        for (k, v) in found_map.into_iter() {
            work_id_by_key.insert(k, v);
        }
        // 未検索キーは None として初期化
        for k in keys.iter() {
            work_id_by_key.entry(k.clone()).or_insert(None);
        }

        // 除外対象 Work ID を取得
        let omitted_work_ids: HashSet<StrId<domain::works::Work>> = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work_omit();
                    let list = repo.list().await?;
                    Ok(list.into_iter().map(|o| o.work_id).collect())
                })
            })
            .await?;

        // EGS ID ごとの既存 Work ID を検索（Work テーブルから直接）
        let egs_ids: Vec<i32> = games
            .iter()
            .filter_map(|g| (ops.egs)(g).map(|e| e.erogamescape_id))
            .collect();
        let mut egs_id_to_work_id: HashMap<i32, StrId<domain::works::Work>> = HashMap::new();
        if !egs_ids.is_empty() {
            let pairs = self
                .manager
                .run(|repos| {
                    let egs_ids = egs_ids.clone();
                    Box::pin(async move {
                        repos
                            .work()
                            .find_work_ids_by_erogamescape_ids(&egs_ids)
                            .await
                    })
                })
                .await?;
            for (egs, wid) in pairs.into_iter() {
                egs_id_to_work_id.insert(egs, wid);
            }
        }

        Ok(BatchSnapshot {
            work_id_by_key,
            omitted_work_ids,
            egs_id_to_work_id,
        })
    }

    /// 個別ゲームに対する同期計画を決定。
    /// - 既に Work が存在すれば SkipExists
    /// - 除外対象 Work なら SkipOmitted
    /// - それ以外は Apply（新規作成/EGS 情報更新）
    pub async fn decide_for_game_generic<P: Clone, K: Clone + std::cmp::Eq + std::hash::Hash>(
        &self,
        snapshot: &BatchSnapshot<K>,
        param: P,
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<PlanDecisionGeneric<K>> {
        let key = (ops.key_from_param)(&param);

        // ストアキーに紐づく Work が既に存在すればスキップ
        let work_id = snapshot.work_id_by_key.get(&key).cloned().unwrap_or(None);
        if work_id.is_some() {
            return Ok(PlanDecisionGeneric::SkipExists);
        }

        // Work が除外対象ならスキップ（work_id が Some の場合のみチェック）
        if let Some(ref work_id) = work_id {
            if snapshot.omitted_work_ids.contains(work_id) {
                return Ok(PlanDecisionGeneric::SkipOmitted);
            }
        }

        Ok(PlanDecisionGeneric::Apply(SyncApplyGeneric {
            key,
            work_id_by_key: work_id,
            work_id_by_erogamescape: (ops.egs)(&param)
                .and_then(|e| snapshot.egs_id_to_work_id.get(&e.erogamescape_id).cloned()),
            gamename: (ops.gamename)(&param).to_string(),
            parent_pack_work_id: (ops.parent_pack_work_id)(&param),
            egs: (ops.egs)(&param).cloned(),
        }))
    }

    /// 同期適用を実行。
    /// - Work の新規作成または既存利用
    /// - ストアマッピングの upsert
    /// - EGS 情報の upsert（あれば）
    /// - 親パック関連付け（DMM の場合）
    pub async fn execute_apply_with_repos_generic<P, K>(
        repos: &R,
        apply: SyncApplyGeneric<K>,
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<()>
    where
        K: Clone + std::cmp::Eq + std::hash::Hash,
    {
        let SyncApplyGeneric {
            key,
            work_id_by_key,
            work_id_by_erogamescape,
            gamename,
            parent_pack_work_id,
            egs,
        } = apply;

        // 既存 Work があれば利用、なければ新規作成
        let work_id = match work_id_by_key.or(work_id_by_erogamescape) {
            Some(work_id) => work_id,
            None => {
                repos
                    .work()
                    .upsert(&domain::works::NewWork::new(gamename.clone()))
                    .await?
            }
        };

        // ストアキーとのマッピングを upsert
        (ops.upsert_store_mapping)(repos, &key, work_id.clone()).await?;

        // EGS 情報があれば Work に upsert
        if let Some(ref egs_info) = egs {
            // 1) EGS マップを upsert
            repos
                .work()
                .upsert_erogamescape_map(work_id.clone(), egs_info.erogamescape_id)
                .await?;

            // 2) EGS 情報を upsert
            let ruby = if egs_info.gamename_ruby.is_empty() {
                gamename.clone()
            } else {
                egs_info.gamename_ruby.clone()
            };
            let info = domain::erogamescape::NewErogamescapeInformation::new(
                egs_info.erogamescape_id,
                ruby,
                egs_info.brandname.clone(),
                egs_info.brandname_ruby.clone(),
                egs_info.sellday.clone(),
                egs_info.is_nukige,
            );
            repos.erogamescape().upsert_information(&info).await?;
        }

        // 親パック関連付け（DMM パックの場合）
        (ops.link_parent_pack_if_needed)(repos, work_id.clone(), parent_pack_work_id).await?;

        Ok(())
    }
}
