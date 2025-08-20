use std::sync::Arc;
use derive_new::new;

use domain::all_game_cache::{AllGameCache, AllGameCacheOne};
use domain::game_matcher::{GameMatcher, Matcher, MatcherConfig, normalize};

/// ファイルパスや名前からゲームを特定するUseCase
#[derive(new)]
pub struct GameIdentifierUseCase {
    matcher: Arc<dyn GameMatcher + Send + Sync>,
}

impl GameIdentifierUseCase {
    /// デフォルト設定でMatcherを使用してGameIdentifierUseCaseを作成
    pub fn with_default_matcher(game_cache: AllGameCache) -> Self {
        // ゲーム名を正規化
        let normalized_cache: AllGameCache = game_cache
            .iter()
            .map(|game| AllGameCacheOne {
                id: game.id,
                gamename: normalize(&game.gamename),
            })
            .collect();
            
        let matcher = Matcher::with_default_config(normalized_cache);
        Self::new(Arc::new(matcher))
    }

    /// カスタム設定でMatcherを使用してGameIdentifierUseCaseを作成
    pub fn with_custom_matcher(game_cache: AllGameCache, config: MatcherConfig) -> Self {
        // ゲーム名を正規化
        let normalized_cache: AllGameCache = game_cache
            .iter()
            .map(|game| AllGameCacheOne {
                id: game.id,
                gamename: normalize(&game.gamename),
            })
            .collect();
            
        let matcher = Matcher::new(normalized_cache, config);
        Self::new(Arc::new(matcher))
    }

    /// ファイルパスからゲームを特定
    pub fn identify_by_filepath(&self, filepath: &str) -> anyhow::Result<Vec<AllGameCacheOne>> {
        // ファイルパスから情報を抽出
        let file_info = domain::game_matcher::extract_file_info(filepath)?;
        
        // 複数の文字列でマッチング（元実装と同じロジック）
        let mut queries = Vec::new();
        
        // ファイル名マッチング（skip_filenameの場合は除外）
        if !file_info.skip_filename {
            queries.push(file_info.filename);
        }
        
        // 親ディレクトリマッチング
        queries.push(file_info.parent_dir);
        
        Ok(self.matcher.find_candidates(&queries))
    }
    
    /// ゲーム名からゲームを特定
    pub fn identify_by_name(&self, game_name: &str) -> anyhow::Result<Vec<AllGameCacheOne>> {
        let normalized_name = normalize(game_name);
        Ok(self.matcher.find_candidates(&[normalized_name]))
    }
    
    /// 最も可能性の高い候補を1つ取得
    pub fn get_most_probable_candidate(&self, filepath: &str) -> anyhow::Result<Option<AllGameCacheOne>> {
        let candidates = self.identify_by_filepath(filepath)?;
        Ok(candidates.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::all_game_cache::AllGameCacheOne;

    fn create_test_cache() -> AllGameCache {
        vec![
            AllGameCacheOne::new(27123, "pieces/渡り鳥のソムニウム".to_string()),
            AllGameCacheOne::new(1, "テストゲーム".to_string()),
            AllGameCacheOne::new(2, "Another Game".to_string()),
        ]
    }

    #[test]
    fn test_identify_by_filepath() {
        let cache = create_test_cache();
        let identifier = GameIdentifierUseCase::with_default_matcher(cache);
        
        let result = identifier.identify_by_filepath("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe").unwrap();
        
        assert!(!result.is_empty());
        assert_eq!(result[0].id, 27123);
    }

    #[test]
    fn test_identify_by_name() {
        let cache = create_test_cache();
        let identifier = GameIdentifierUseCase::with_default_matcher(cache);
        
        let result = identifier.identify_by_name("pieces").unwrap();
        
        assert!(!result.is_empty());
        assert_eq!(result[0].id, 27123);
    }

    #[test]
    fn test_get_most_probable_candidate() {
        let cache = create_test_cache();
        let identifier = GameIdentifierUseCase::with_default_matcher(cache);
        
        let result = identifier.get_most_probable_candidate("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe").unwrap();
        
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 27123);
    }

    #[test]
    fn test_skip_filename_logic() {
        let cache = create_test_cache();
        let identifier = GameIdentifierUseCase::with_default_matcher(cache);
        
        // "game.exe"のような汎用的な名前では、ファイル名マッチングがスキップされる
        let result = identifier.identify_by_filepath("C:\\Program Files\\TestGame\\game.exe");
        
        // エラーにならず、親ディレクトリでマッチングが試行される
        assert!(result.is_ok());
    }
}