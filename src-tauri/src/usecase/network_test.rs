#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        domain::{explorer::network::MockNetworkExplorer, network::ErogamescapeIDNamePair},
        infrastructure::explorermock::MockExplorersExtMock,
        usecase::network::NetworkUseCase,
    };

    fn create_test_game_pairs() -> Vec<ErogamescapeIDNamePair> {
        vec![
            ErogamescapeIDNamePair {
                id: 1,
                gamename: "Test Game 1".to_string(),
            },
            ErogamescapeIDNamePair {
                id: 2,
                gamename: "Test Game 2".to_string(),
            },
            ErogamescapeIDNamePair {
                id: 3,
                gamename: "Test Game 3".to_string(),
            },
        ]
    }

    #[tokio::test]
    async fn test_get_all_games_success() {
        let mut mock_network_explorer = MockNetworkExplorer::new();
        let expected_games = create_test_game_pairs();
        mock_network_explorer
            .expect_get_all_games()
            .times(1)
            .returning(move || Ok(expected_games.clone()));

        let mut mock_explorers = MockExplorersExtMock::new();
        mock_explorers
            .expect_network_explorer()
            .return_const(mock_network_explorer);

        let use_case = NetworkUseCase::new(Arc::new(mock_explorers));

        let result = use_case.get_all_games().await;
        assert!(result.is_ok());
        let games = result.unwrap();
        assert_eq!(games.len(), 3);
        assert_eq!(games[0].id, 1);
        assert_eq!(games[0].gamename, "Test Game 1");
        assert_eq!(games[1].id, 2);
        assert_eq!(games[1].gamename, "Test Game 2");
        assert_eq!(games[2].id, 3);
        assert_eq!(games[2].gamename, "Test Game 3");
    }

    #[tokio::test]
    async fn test_get_all_games_empty_result() {
        let mut mock_network_explorer = MockNetworkExplorer::new();
        mock_network_explorer
            .expect_get_all_games()
            .times(1)
            .returning(|| Ok(vec![]));

        let mut mock_explorers = MockExplorersExtMock::new();
        mock_explorers
            .expect_network_explorer()
            .return_const(mock_network_explorer);

        let use_case = NetworkUseCase::new(Arc::new(mock_explorers));

        let result = use_case.get_all_games().await;
        assert!(result.is_ok());
        let games = result.unwrap();
        assert!(games.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_games_error_propagation() {
        let mut mock_network_explorer = MockNetworkExplorer::new();
        mock_network_explorer
            .expect_get_all_games()
            .times(1)
            .returning(|| Err(anyhow::anyhow!("Network error")));

        let mut mock_explorers = MockExplorersExtMock::new();
        mock_explorers
            .expect_network_explorer()
            .return_const(mock_network_explorer);

        let use_case = NetworkUseCase::new(Arc::new(mock_explorers));

        let result = use_case.get_all_games().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Network error"));
    }

    // 外部ネットワークリクエストを伴う実際の統合テスト
    // 通常の実行では実行されない（#[ignore]属性）
    #[tokio::test]
    #[ignore = "外部ネットワークリクエストを伴うため、通常のテスト実行ではスキップ"]
    async fn test_get_all_games_actual_network_request() {
        use crate::infrastructure::explorerimpl::explorer::Explorers;

        let explorers = Explorers::new();
        let use_case = NetworkUseCase::new(Arc::new(explorers));

        let result = use_case.get_all_games().await;

        // このテストは実際のネットワークリクエストを行うため、
        // 結果は環境に依存する。エラーが発生する可能性もある。
        match result {
            Ok(games) => {
                println!("取得したゲーム数: {}", games.len());
                if !games.is_empty() {
                    println!("最初のゲーム: {:?}", games[0]);
                }
            }
            Err(e) => {
                println!("ネットワークエラー: {}", e);
                // ネットワークエラーは想定される範囲内なので、パニックしない
            }
        }
    }

    #[tokio::test]
    async fn test_get_all_games_large_dataset() {
        let mut mock_network_explorer = MockNetworkExplorer::new();
        let large_dataset: Vec<ErogamescapeIDNamePair> = (1..=1000)
            .map(|i| ErogamescapeIDNamePair {
                id: i,
                gamename: format!("Test Game {}", i),
            })
            .collect();

        mock_network_explorer
            .expect_get_all_games()
            .times(1)
            .returning(move || Ok(large_dataset.clone()));

        let mut mock_explorers = MockExplorersExtMock::new();
        mock_explorers
            .expect_network_explorer()
            .return_const(mock_network_explorer);

        let use_case = NetworkUseCase::new(Arc::new(mock_explorers));

        let result = use_case.get_all_games().await;
        assert!(result.is_ok());
        let games = result.unwrap();
        assert_eq!(games.len(), 1000);
        assert_eq!(games[0].id, 1);
        assert_eq!(games[999].id, 1000);
    }
}
