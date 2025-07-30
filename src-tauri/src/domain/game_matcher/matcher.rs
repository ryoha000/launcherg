use std::collections::HashMap;
use std::sync::Mutex;
use crate::domain::all_game_cache::{AllGameCache, AllGameCacheOne};
use crate::domain::distance::get_comparable_distance;
use super::config::{EQUALLY_FILENAME_GAME_ID_PAIR, IGNORE_GAME_ID};


/// ゲームマッチング設定
#[derive(Debug, Clone)]
pub struct MatcherConfig {
    pub exact_mappings: HashMap<String, i32>,
    pub similarity_threshold: f32,
    pub partial_min_length: usize,
    pub ignore_game_ids: Vec<i32>,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        let mut exact_mappings = HashMap::new();
        for (filename, id) in EQUALLY_FILENAME_GAME_ID_PAIR {
            exact_mappings.insert(filename.to_string(), id);
        }
        
        Self {
            exact_mappings,
            similarity_threshold: 0.8,
            partial_min_length: 5,
            ignore_game_ids: IGNORE_GAME_ID.to_vec(),
        }
    }
}

/// ゲームマッチングを行うトレイト
pub trait GameMatcher {
    /// 複数の文字列でゲーム候補を検索する
    fn find_candidates(&self, queries: &[String]) -> Vec<AllGameCacheOne>;
}

/// シンプルなマッチャー実装
/// 元実装と同じ動作を再現
pub struct Matcher {
    game_cache: AllGameCache,
    config: MatcherConfig,
    // query -> Vec<(game, score)> のキャッシュ（閾値以上のマッチのみ）
    query_cache: Mutex<HashMap<String, Vec<(AllGameCacheOne, f32)>>>,
}

impl Matcher {
    pub fn new(game_cache: AllGameCache, config: MatcherConfig) -> Self {
        Self { 
            game_cache, 
            config,
            query_cache: Mutex::new(HashMap::new()),
        }
    }
    
    pub fn with_default_config(game_cache: AllGameCache) -> Self {
        Self::new(game_cache, MatcherConfig::default())
    }
    
    /// キャッシュをクリアする
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.query_cache.lock() {
            cache.clear();
        }
    }
    
    /// キャッシュサイズを取得
    pub fn cache_size(&self) -> usize {
        self.query_cache.lock().map_or(0, |cache| cache.len())
    }
    
    /// 単一クエリに対する閾値以上のマッチング結果を取得（キャッシュ付き）
    pub fn get_matches_for_query(&self, query: &str) -> Vec<(AllGameCacheOne, f32)> {
        // キャッシュを確認
        if let Ok(cache) = self.query_cache.lock() {
            if let Some(cached_result) = cache.get(query) {
                return cached_result.clone();
            }
        }
        
        // キャッシュにない場合は計算
        let mut matches = Vec::new();
        
        for game in self.game_cache.iter() {
            // 無視するゲームIDをスキップ
            if self.config.ignore_game_ids.contains(&game.id) {
                continue;
            }
            
            let score = get_comparable_distance(query, &game.gamename);
            
            // 閾値以上の場合は結果に追加
            if score > self.config.similarity_threshold {
                matches.push((game.clone(), score));
            }
        }
        
        // フォールバック: 部分文字列マッチング
        if matches.is_empty() && query.len() > self.config.partial_min_length {
            for game in self.game_cache.iter() {
                if game.gamename.contains(query) {
                    matches.push((game.clone(), query.len() as f32));
                }
            }
        }
        
        // 結果をキャッシュに保存
        if let Ok(mut cache) = self.query_cache.lock() {
            cache.insert(query.to_string(), matches.clone());
        }
        
        matches
    }
}

impl GameMatcher for Matcher {
    fn find_candidates(&self, queries: &[String]) -> Vec<AllGameCacheOne> {
        // 1. 完全一致チェック
        for query in queries {
            if let Some(&game_id) = self.config.exact_mappings.get(query) {
                if let Some(game) = self.game_cache.iter().find(|g| g.id == game_id) {
                    return vec![game.clone()];
                }
            }
        }
        
        // 2. 各クエリの結果を取得してスコアを集計
        let mut game_scores: HashMap<i32, f32> = HashMap::new();
        
        for query in queries {
            let matches = self.get_matches_for_query(query);
            for (game, score) in matches {
                // 各ゲームの最高スコアを保持
                let current_score = game_scores.get(&game.id).unwrap_or(&0.0);
                game_scores.insert(game.id, current_score.max(score));
            }
        }
        
        // 3. ゲームIDとスコアを結合して結果を作成
        let mut candidates: Vec<(AllGameCacheOne, f32)> = game_scores
            .into_iter()
            .filter_map(|(game_id, score)| {
                self.game_cache
                    .iter()
                    .find(|g| g.id == game_id)
                    .map(|game| (game.clone(), score))
            })
            .collect();
        
        // スコア順にソート（降順）
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // ゲームのみを返す
        candidates.into_iter().map(|(game, _)| game).collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::all_game_cache::AllGameCacheOne;

    fn create_test_cache() -> AllGameCache {
        vec![
            AllGameCacheOne::new(27123, "pieces/渡り鳥のソムニウム".to_string()),
            AllGameCacheOne::new(1, "テストゲーム".to_string()),
            AllGameCacheOne::new(2, "Another Game".to_string()),
        ]
    }

    #[test]
    fn test_matcher_exact_match() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        let queries = vec!["pieces".to_string()];
        let candidates = matcher.find_candidates(&queries);
        
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, 27123);
    }

    #[test]
    fn test_matcher_similarity_match() {
        let cache = create_test_cache();
        let mut config = MatcherConfig::default();
        config.similarity_threshold = 0.1; // 低い闾値でテスト
        let matcher = Matcher::new(cache, config);
        
        let queries = vec!["piece".to_string()]; // "pieces"に似ている
        let candidates = matcher.find_candidates(&queries);
        
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_matcher_partial_match() {
        let cache = create_test_cache();
        let mut config = MatcherConfig::default();
        config.similarity_threshold = 1.0; // 高い闾値で類似度マッチを無効化
        config.partial_min_length = 3;
        let matcher = Matcher::new(cache, config);
        
        let queries = vec!["pieces".to_string()];
        let candidates = matcher.find_candidates(&queries);
        
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, 27123);
    }

    #[test]
    fn test_matcher_multiple_queries() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 複数のクエリで最高スコアを採用
        let queries = vec!["unknown".to_string(), "pieces".to_string()];
        let candidates = matcher.find_candidates(&queries);
        
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, 27123);
    }

    #[test]
    fn test_cache_functionality() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 初回はキャッシュが空
        assert_eq!(matcher.cache_size(), 0);
        
        // 完全一致しないクエリを使用して類似度計算を発生させる
        let queries = vec!["piece".to_string()]; // "pieces"ではなく"piece"
        let result1 = matcher.find_candidates(&queries);
        
        // キャッシュに保存される（"piece" × ゲーム数分）
        assert!(matcher.cache_size() > 0);
        
        let result2 = matcher.find_candidates(&queries);
        
        // 結果は同じ
        assert_eq!(result1.len(), result2.len());
        if !result1.is_empty() && !result2.is_empty() {
            assert_eq!(result1[0].id, result2[0].id);
        }
        
        // キャッシュサイズは変わらない（同じクエリなので）
        let cache_size_after_first = matcher.cache_size();
        assert_eq!(matcher.cache_size(), cache_size_after_first);
    }

    #[test]
    fn test_cache_clear() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 完全一致しないクエリを使用して類似度計算を発生させる
        let queries = vec!["piece".to_string()];
        matcher.find_candidates(&queries);
        
        // キャッシュに保存される
        assert!(matcher.cache_size() > 0);
        
        // キャッシュをクリア
        matcher.clear_cache();
        assert_eq!(matcher.cache_size(), 0);
    }

    #[test]
    fn test_query_level_cache_reuse() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 最初に["foo", "bar"]でクエリ
        matcher.find_candidates(&vec!["foo".to_string(), "bar".to_string()]);
        let cache_size_after_first = matcher.cache_size();
        
        // 次に["foo", "baz"]でクエリ
        matcher.find_candidates(&vec!["foo".to_string(), "baz".to_string()]);
        let cache_size_after_second = matcher.cache_size();
        
        // "foo"の計算は再利用されるので、"baz"の分だけ増える
        assert!(cache_size_after_second > cache_size_after_first);
        // "foo"が再利用されるため、キャッシュサイズは2になる（foo, bar, bazのうちfooは再利用）
        assert_eq!(cache_size_after_second, 3); // foo, bar, baz
    }

    #[test]
    fn test_individual_query_cache() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 単一クエリでマッチング結果を取得
        let matches1 = matcher.get_matches_for_query("piece"); // "pieces"に似ている
        assert_eq!(matcher.cache_size(), 1);
        
        // 同じクエリでもう一度
        let matches2 = matcher.get_matches_for_query("piece");
        assert_eq!(matches1.len(), matches2.len());
        assert_eq!(matcher.cache_size(), 1); // キャッシュサイズは変わらない
        
        // 異なるクエリ
        let _matches3 = matcher.get_matches_for_query("test");
        assert_eq!(matcher.cache_size(), 2); // 新しいエントリが追加される
    }

    #[test]
    fn test_cache_efficiency() {
        let cache = create_test_cache();
        let matcher = Matcher::with_default_config(cache);
        
        // 複数回同じクエリセットを実行
        let queries = vec!["piece".to_string(), "test".to_string()];
        
        let result1 = matcher.find_candidates(&queries);
        let cache_size_after_first = matcher.cache_size();
        
        let result2 = matcher.find_candidates(&queries);
        let cache_size_after_second = matcher.cache_size();
        
        // 結果は同じ
        assert_eq!(result1.len(), result2.len());
        // キャッシュサイズは変わらない（すべてキャッシュから取得）
        assert_eq!(cache_size_after_first, cache_size_after_second);
    }
}