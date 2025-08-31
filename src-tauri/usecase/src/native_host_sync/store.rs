use std::future::Future;
use std::pin::Pin;
use std::collections::{HashMap, HashSet};

use domain::Id;

use super::*;

#[derive(Clone)]
pub struct StoreOps<P, K, R>
where
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub key_from_param: fn(&P) -> K,
    pub gamename: fn(&P) -> &str,
    pub image_url: fn(&P) -> &str,
    pub egs: fn(&P) -> Option<&EgsInfo>,
    pub parent_pack_work_id: fn(&P) -> Option<Id<domain::works::Work>>,

    pub find_work_id_by_key: for<'a> fn(&'a R, &'a K) -> Pin<Box<dyn Future<Output=anyhow::Result<Option<Id<domain::works::Work>>>> + Send + 'a>>,
    pub upsert_store_mapping: for<'a> fn(&'a R, &'a K, Id<domain::works::Work>) -> Pin<Box<dyn Future<Output=anyhow::Result<()>> + Send + 'a>>,
    pub enqueue_images_with_repos: for<'a> fn(&'a R, &'a dyn SavePathResolver, &'a domain::Id<domain::collection::CollectionElement>, &'a K, &'a str) -> Pin<Box<dyn Future<Output=anyhow::Result<()>> + Send + 'a>>,
    pub link_parent_pack_if_needed: for<'a> fn(&'a R, Id<domain::works::Work>, Option<Id<domain::works::Work>>) -> Pin<Box<dyn Future<Output=anyhow::Result<()>> + Send + 'a>>,
}

#[derive(Clone, Debug)]
pub struct BatchSnapshot<K> {
    pub work_id_by_key: HashMap<K, Option<Id<domain::works::Work>>>,
    pub mapped_keys: HashMap<K, Id<domain::collection::CollectionElement>>,
    pub omitted_work_ids: HashSet<Id<domain::works::Work>>,
    pub egs_id_to_collection_id: HashMap<i32, Id<domain::collection::CollectionElement>>,
    pub egs_id_to_work_id: HashMap<i32, Id<domain::works::Work>>,
}

#[derive(Clone, Debug)]
pub struct SyncApplyGeneric<K> {
    pub key: K,
    pub work_id_by_key: Option<Id<domain::works::Work>>,
    pub work_id_by_erogamescape: Option<Id<domain::works::Work>>,
    pub collection_element_id_by_erogamescape: Option<Id<domain::collection::CollectionElement>>,
    pub gamename: String,
    pub image_url: String,
    pub parent_pack_work_id: Option<Id<domain::works::Work>>,
    pub egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
pub enum PlanDecisionGeneric<K> {
    SkipExists,
    SkipOmitted,
    Apply(SyncApplyGeneric<K>),
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn build_batch_snapshot<P, K>(
        &self,
        games: &[P],
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<BatchSnapshot<K>>
    where
        K: Clone + std::cmp::Eq + std::hash::Hash + Send + Sync + 'static,
    {
        // keys
        let keys: Vec<K> = games.iter().map(|g| (ops.key_from_param)(g)).collect();

        // work_id_by_key
        let mut work_id_by_key: HashMap<K, Option<Id<domain::works::Work>>> = HashMap::new();
        let found_map = self.manager.run(|repos| {
            let keys = keys.clone();
            let ops = ops;
            Box::pin(async move {
                let mut out = HashMap::new();
                for k in keys.iter() {
                    let wid = (ops.find_work_id_by_key)(&repos, k).await?;
                    out.insert(k.clone(), wid);
                }
                Ok::<HashMap<K, Option<Id<domain::works::Work>>>, anyhow::Error>(out)
            })
        }).await?;
        for (k, v) in found_map.into_iter() { work_id_by_key.insert(k, v); }
        for k in keys.iter() { work_id_by_key.entry(k.clone()).or_insert(None); }

        // mapped_keys via work_ids
        let work_ids: Vec<Id<domain::works::Work>> = keys.iter().filter_map(|k| work_id_by_key.get(k).and_then(|v| v.clone())).collect();
        let mut mapped_keys: HashMap<K, Id<domain::collection::CollectionElement>> = HashMap::new();
        if !work_ids.is_empty() {
            let rows = self.manager.run(|repos| {
                let work_ids = work_ids.clone();
                Box::pin(async move {
                    let mut repo = repos.collection();
                    repo.get_collection_ids_by_work_ids(&work_ids).await
                })
            }).await?;
            let mut keys_by_work: HashMap<Id<domain::works::Work>, Vec<K>> = HashMap::new();
            for (k, v) in work_id_by_key.iter() { if let Some(wid) = v.clone() { keys_by_work.entry(wid).or_default().push(k.clone()); } }
            for (wid, ce) in rows.into_iter() { if let Some(kk) = keys_by_work.get(&wid) { for k in kk.iter() { mapped_keys.insert(k.clone(), ce.clone()); } } }
        }

        // omitted
        let omitted_work_ids: HashSet<Id<domain::works::Work>> = self.manager.run(|repos| {
            Box::pin(async move {
                let mut repo = repos.work_omit();
                let list = repo.list().await?;
                Ok(list.into_iter().map(|o| o.work_id).collect())
            })
        }).await?;

        // egs
        let egs_ids: Vec<i32> = games.iter().filter_map(|g| (ops.egs)(g).map(|e| e.erogamescape_id)).collect();
        let mut egs_id_to_collection_id: HashMap<i32, Id<domain::collection::CollectionElement>> = HashMap::new();
        let mut egs_id_to_work_id: HashMap<i32, Id<domain::works::Work>> = HashMap::new();
        if !egs_ids.is_empty() {
            let rows = self.manager.run(|repos| {
                let egs_ids = egs_ids.clone();
                Box::pin(async move {
                    let mut repo = repos.collection();
                    repo.get_collection_ids_by_erogamescape_ids(&egs_ids).await
                })
            }).await?;
            for (egs_id, ceid) in rows.into_iter() { egs_id_to_collection_id.insert(egs_id, ceid); }

            let ce_ids: Vec<i32> = egs_id_to_collection_id.values().map(|id| id.value).collect();
            if !ce_ids.is_empty() {
                let rows = self.manager.run(|repos| {
                    let ce_ids = ce_ids.clone();
                    Box::pin(async move {
                        let mut repo = repos.collection();
                        repo.get_work_ids_by_collection_ids(&ce_ids).await
                    })
                }).await?;
                let mut first_work_by_ce: HashMap<i32, Id<domain::works::Work>> = HashMap::new();
                for (ceid, wid) in rows.into_iter() { first_work_by_ce.entry(ceid.value).or_insert(wid); }
                for (egs_id, ceid) in egs_id_to_collection_id.iter() { if let Some(wid) = first_work_by_ce.get(&ceid.value) { egs_id_to_work_id.insert(*egs_id, wid.clone()); } }
            }
        }

        Ok(BatchSnapshot { work_id_by_key, mapped_keys, omitted_work_ids, egs_id_to_collection_id, egs_id_to_work_id })
    }

    pub async fn decide_for_game_generic<P: Clone, K: Clone + std::cmp::Eq + std::hash::Hash>(
        &self,
        snapshot: &BatchSnapshot<K>,
        param: P,
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<PlanDecisionGeneric<K>> {
        let key = (ops.key_from_param)(&param);
        if snapshot.mapped_keys.contains_key(&key) { return Ok(PlanDecisionGeneric::SkipExists); }

        let work_id = snapshot.work_id_by_key.get(&key).cloned().unwrap_or(None);
        if let Some(ref work_id) = work_id { if snapshot.omitted_work_ids.contains(work_id) { return Ok(PlanDecisionGeneric::SkipOmitted); } }

        Ok(PlanDecisionGeneric::Apply(SyncApplyGeneric {
            key,
            work_id_by_key: work_id,
            work_id_by_erogamescape: (ops.egs)(&param).and_then(|e| snapshot.egs_id_to_work_id.get(&e.erogamescape_id).cloned()),
            collection_element_id_by_erogamescape: (ops.egs)(&param).and_then(|e| snapshot.egs_id_to_collection_id.get(&e.erogamescape_id).cloned()),
            gamename: (ops.gamename)(&param).to_string(),
            image_url: (ops.image_url)(&param).to_string(),
            parent_pack_work_id: (ops.parent_pack_work_id)(&param),
            egs: (ops.egs)(&param).cloned(),
        }))
    }

    pub async fn execute_apply_with_repos_generic<P, K>(
        repos: &R,
        apply: SyncApplyGeneric<K>,
        resolver: &dyn SavePathResolver,
        ops: &StoreOps<P, K, R>,
    ) -> anyhow::Result<()>
    where
        K: Clone + std::cmp::Eq + std::hash::Hash,
    {
        let SyncApplyGeneric { key, work_id_by_key, work_id_by_erogamescape, collection_element_id_by_erogamescape, gamename, image_url, parent_pack_work_id, egs } = apply;

        // Work 確保
        let work_id = match work_id_by_key.or(work_id_by_erogamescape) {
            Some(work_id) => work_id,
            None => repos.work().upsert(&domain::works::NewWork::new(gamename.clone())).await?,
        };
        // ストアキー upsert
        (ops.upsert_store_mapping)(repos, &key, work_id.clone()).await?;

        // CE 確保
        let collection_element_id = match collection_element_id_by_erogamescape {
            Some(ceid) => ceid,
            None => {
                let ceid = repos.collection().allocate_new_collection_element_id(&gamename).await?;
                if let Some(egs_info) = egs.as_ref() {
                    repos.collection().upsert_erogamescape_map(&ceid, egs_info.erogamescape_id).await?;
                    repos.collection().upsert_collection_element_info(&domain::collection::NewCollectionElementInfo::new(ceid.clone(), gamename.clone(), egs_info.brandname.clone(), egs_info.brandname_ruby.clone(), egs_info.sellday.clone(), egs_info.is_nukige)).await?;
                }
                ceid
            }
        };

        // CE-Work マッピング
        repos.collection().upsert_work_mapping(&collection_element_id, work_id.clone()).await?;

        // 親パック（DMMのみ有効）
        (ops.link_parent_pack_if_needed)(repos, work_id.clone(), parent_pack_work_id).await?;

        // 画像投入
        (ops.enqueue_images_with_repos)(repos, resolver, &collection_element_id, &key, &image_url).await?;

        Ok(())
    }
}


