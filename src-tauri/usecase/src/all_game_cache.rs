use std::sync::Arc;

use chrono::{DateTime, Local};

use domain::all_game_cache::AllGameCacheOne as MatcherAllGameCacheOne;
use domain::all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne};
use domain::game_matcher::GameMatcher;
use domain::repository::{
    all_game_cache::AllGameCacheRepository, manager::RepositoryManager, RepositoriesExt,
};
use std::marker::PhantomData;

pub struct AllGameCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: PhantomData<R>,
    matcher: Arc<dyn GameMatcher + Send + Sync>,
}

impl<M, R> AllGameCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    /// 既存テスト互換のためのデフォルトコンストラクタ（NopMatcher を使用）
    pub fn new(manager: Arc<M>) -> Self {
        Self {
            manager,
            _marker: PhantomData,
            matcher: Arc::new(NopMatcher),
        }
    }

    /// 明示的に Matcher を注入して生成
    pub fn with_matcher(manager: Arc<M>, matcher: Arc<dyn GameMatcher + Send + Sync>) -> Self {
        Self {
            manager,
            _marker: PhantomData,
            matcher,
        }
    }
    pub async fn get(&self, id: i32) -> anyhow::Result<Option<AllGameCacheOneWithThumbnailUrl>> {
        self.manager
            .run(move |repos| {
                Box::pin(async move {
                    Ok(repos
                        .all_game_cache()
                        .get_by_ids(vec![id])
                        .await?
                        .first()
                        .cloned())
                })
            })
            .await
    }
    pub async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        self.manager
            .run(move |repos| Box::pin(async move { repos.all_game_cache().get_by_ids(ids).await }))
            .await
    }
    pub async fn get_all_game_cache(&self) -> anyhow::Result<AllGameCache> {
        self.manager
            .run(|repos| Box::pin(async move { repos.all_game_cache().get_all().await }))
            .await
    }
    pub async fn get_cache_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)> {
        self.manager
            .run(|repos| Box::pin(async move { repos.all_game_cache().get_last_updated().await }))
            .await
    }
    pub async fn update_all_game_cache(
        &self,
        cache: Vec<NewAllGameCacheOne>,
    ) -> anyhow::Result<()> {
        if cache.is_empty() {
            return Ok(());
        }
        let ids: Vec<i32> = cache.iter().map(|v| v.id).collect();
        self.manager
            .run_in_transaction(move |repos| {
                let cache_clone = cache.clone();
                Box::pin(async move {
                    repos.all_game_cache().delete_by_ids(ids).await?;
                    repos.all_game_cache().update(cache_clone).await
                })
            })
            .await?;

        // 変更を GameMatcher へ反映
        let all = self.get_all_game_cache().await?;
        let passthrough: Vec<MatcherAllGameCacheOne> = all
            .into_iter()
            .map(|g| MatcherAllGameCacheOne::new(g.id, g.gamename))
            .collect();
        self.matcher.update_all_game_cache(passthrough);
        Ok(())
    }

    pub async fn search_games_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let name = name.to_string();
        self.manager
            .run(move |repos| {
                Box::pin(async move { repos.all_game_cache().search_by_name(&name).await })
            })
            .await
    }
}

/// 何もしないデフォルト Matcher（テスト・後方互換用）
struct NopMatcher;

impl GameMatcher for NopMatcher {
    fn find_candidates(
        &self,
        _queries: &[String],
    ) -> Vec<(domain::all_game_cache::AllGameCacheOne, f32)> {
        Vec::new()
    }
    fn update_all_game_cache(&self, _new_cache: domain::all_game_cache::AllGameCache) {}
}
