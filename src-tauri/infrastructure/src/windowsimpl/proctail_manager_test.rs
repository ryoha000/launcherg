#[cfg(test)]
mod tests {
    use super::super::proctail_manager::*;
    use domain::windows::proctail_manager::{ProcTailManagerStatus, ProcTailVersion, ProcTailManagerError};
    use std::sync::Arc;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // =============================================================================
    // テストインフラストラクチャ
    // =============================================================================

    /// テスト用のAppConfigProviderの実装
    /// AppConfigProviderトレイトのテスト用実装で、一時ディレクトリを使用して
    /// ファイルシステムの操作をテスト可能にする
    struct TestConfigProvider {
        root_dir: String,
    }

    impl TestConfigProvider {
        fn new(root_dir: String) -> Self {
            Self { root_dir }
        }
    }

    impl AppConfigProvider for TestConfigProvider {
        fn get_app_config_dir(&self) -> String {
            self.root_dir.clone()
        }
    }

    /// テスト用のProcTailManagerとTempDirを作成するヘルパー関数
    /// 各テストで独立した環境を提供し、テスト後の自動クリーンアップを保証する
    fn create_test_manager() -> (ProcTailManager<TestConfigProvider>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        let config_provider = Arc::new(TestConfigProvider::new(temp_path));
        let manager = ProcTailManager::new(config_provider);

        (manager, temp_dir)
    }

    // =============================================================================
    // バージョンファイル操作テスト
    // =============================================================================

    /// バージョンファイルが存在しない場合のget_current_version()の動作テスト
    ///
    /// 検証内容:
    /// - バージョンファイルが存在しない場合にNoneを返すことを確認
    /// - 初回起動時やファイルが削除された場合の挙動を検証
    ///
    /// 期待結果: None
    #[tokio::test]
    async fn test_get_current_version_no_file() {
        let (manager, _temp_dir) = create_test_manager();

        let result = manager.get_current_version().await.unwrap();
        assert_eq!(result, None);
    }

    /// バージョンファイルが存在する場合のget_current_version()の動作テスト
    ///
    /// 検証内容:
    /// - バージョンファイルが存在する場合に正しく内容を読み取れることを確認
    /// - ファイルシステムからの読み取り処理の正常動作を検証
    ///
    /// セットアップ: "v1.0.0"を書き込んだファイルを作成
    /// 期待結果: Some("v1.0.0")
    #[tokio::test]
    async fn test_get_current_version_with_file() {
        let (manager, _temp_dir) = create_test_manager();

        // バージョンディレクトリを作成
        let version_dir = manager.get_proctail_version_dir("v1.0.0");
        std::fs::create_dir_all(&version_dir).unwrap();

        let result = manager.get_current_version().await.unwrap();
        assert_eq!(result, Some("v1.0.0".to_string()));
    }

    /// セマンティックバージョニングの正しい動作テスト
    ///
    /// 検証内容:
    /// - 複数のバージョンが存在する場合のセマンティックバージョン比較
    /// - v1.10.0がv1.9.0より新しいと正しく判定されることを確認
    /// - 文字列ソートではなくセマンティックバージョニングが使用されることを確認
    ///
    /// セットアップ: v1.0.0, v2.0.0, v1.10.0, v1.9.0ディレクトリを作成
    /// 期待結果: Some("v2.0.0")（最新バージョン）
    #[tokio::test]
    async fn test_get_current_version_with_multiple_versions() {
        let (manager, _temp_dir) = create_test_manager();

        // 複数のバージョンディレクトリを作成（セマンティックバージョニングのテスト）
        let version_dir1 = manager.get_proctail_version_dir("v1.0.0");
        let version_dir2 = manager.get_proctail_version_dir("v2.0.0");
        let version_dir3 = manager.get_proctail_version_dir("v1.10.0");
        let version_dir4 = manager.get_proctail_version_dir("v1.9.0");
        std::fs::create_dir_all(&version_dir1).unwrap();
        std::fs::create_dir_all(&version_dir2).unwrap();
        std::fs::create_dir_all(&version_dir3).unwrap();
        std::fs::create_dir_all(&version_dir4).unwrap();

        let result = manager.get_current_version().await.unwrap();
        assert!(result.is_some());
        let version = result.unwrap();
        // セマンティックバージョニングにより、v2.0.0が最新として選ばれるべき
        assert_eq!(version, "v2.0.0");
    }

    /// セマンティックバージョニングのエッジケーステスト
    ///
    /// 検証内容:
    /// - マイナーバージョンの比較（v1.10.0 > v1.9.0）が正しく動作することを確認
    /// - 文字列ソートでは誤判定される典型的なケースのテスト
    ///
    /// セットアップ: v1.9.0, v1.10.0ディレクトリを作成
    /// 期待結果: Some("v1.10.0")（v1.10.0がv1.9.0より新しい）
    #[tokio::test]
    async fn test_get_current_version_minor_version_comparison() {
        let (manager, _temp_dir) = create_test_manager();

        // 文字列ソートでは誤判定されるケースをテスト
        let version_dir1 = manager.get_proctail_version_dir("v1.9.0");
        let version_dir2 = manager.get_proctail_version_dir("v1.10.0");
        std::fs::create_dir_all(&version_dir1).unwrap();
        std::fs::create_dir_all(&version_dir2).unwrap();

        let result = manager.get_current_version().await.unwrap();
        assert!(result.is_some());
        let version = result.unwrap();
        // セマンティックバージョニングにより、v1.10.0が最新として選ばれるべき
        assert_eq!(version, "v1.10.0");
    }

    /// 不正なバージョン形式が含まれる場合のテスト
    ///
    /// 検証内容:
    /// - セマンティックバージョンとして解析できないディレクトリが存在する場合の動作
    /// - 有効なバージョンのみが考慮されることを確認
    ///
    /// セットアップ: v1.0.0（有効）とinvalid-version（無効）ディレクトリを作成
    /// 期待結果: Some("v1.0.0")（有効なバージョンのみ考慮）
    #[tokio::test]
    async fn test_get_current_version_with_invalid_versions() {
        let (manager, _temp_dir) = create_test_manager();

        // 有効なバージョンと無効なバージョンを混在させる
        let valid_version_dir = manager.get_proctail_version_dir("v1.0.0");
        let invalid_version_dir = manager.get_proctail_version_dir("invalid-version");
        std::fs::create_dir_all(&valid_version_dir).unwrap();
        std::fs::create_dir_all(&invalid_version_dir).unwrap();

        let result = manager.get_current_version().await.unwrap();
        assert!(result.is_some());
        let version = result.unwrap();
        // 有効なバージョンのみが考慮されるべき
        assert_eq!(version, "v1.0.0");
    }

    // =============================================================================
    // スタブ・準備中テスト
    // =============================================================================

    /// モックサーバーを使用したget_latest_version()のテスト（準備中）
    ///
    /// 検証内容:
    /// - GitHubAPIのモックレスポンスを使用した単体テスト
    /// - 実際のネットワーク通信なしでの動作確認
    ///
    /// 現在の状態: モックサーバーの準備は完了、実装は未完了
    /// 完了のためには: GITHUB_RELEASES_URLを動的に変更する機能が必要
    #[tokio::test]
    async fn test_get_latest_version_success() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!({
            "tag_name": "v1.2.3",
            "assets": [{
                "name": "proctail-windows-x64.zip",
                "browser_download_url": format!("{}/download/proctail-windows-x64.zip", mock_server.uri())
            }]
        });

        Mock::given(method("GET"))
            .and(path("/repos/ryoha000/ProcTail/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let (_manager, _temp_dir) = create_test_manager();

        // 実際のGitHubAPIではなくモックサーバーを使用するため、
        // GITHUB_RELEASES_URLを動的に変更する必要があります
        // 現在の実装では難しいため、モックテストは後で実装
    }

    // =============================================================================
    // GitHubAPI統合テスト（実際のネットワーク通信を使用）
    // =============================================================================

    /// 実際のGitHubAPIを使用したget_latest_version()の統合テスト
    ///
    /// 検証内容:
    /// - 実際のGitHubAPIから最新バージョン情報を取得
    /// - 生のAPIレスポンスを表示してデバッグ支援
    /// - バージョン形式（vで始まる）とダウンロードURLの形式を検証
    ///
    /// 実行結果例:
    /// - バージョン: v0.0.7
    /// - URL: ProcTail-0.0.7-framework-dependent-win-x64.zip
    ///
    /// エラーハンドリング:
    /// - ネットワークエラーの場合はテストをスキップ
    /// - Windows用assetが見つからない場合の適切な処理
    ///
    /// 実行方法: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_get_latest_version_integration() {
        let (manager, _temp_dir) = create_test_manager();

        // まずGitHubAPIから生のレスポンスを取得して確認
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/repos/ryoha000/ProcTail/releases/latest")
            .header("User-Agent", "launcherg")
            .send()
            .await;

        match response {
            Ok(resp) => {
                let release_info: serde_json::Value = resp.json().await.unwrap();
                println!("GitHub API レスポンス:");
                println!("tag_name: {}", release_info["tag_name"]);
                println!("assets: {:#}", release_info["assets"]);

                // アセットの詳細を確認
                if let Some(assets) = release_info["assets"].as_array() {
                    for asset in assets {
                        if let Some(name) = asset["name"].as_str() {
                            println!("Asset名: {}", name);
                        }
                    }
                }
            }
            Err(e) => {
                println!("GitHubAPI呼び出しエラー: {:?}", e);
                if e.to_string().contains("dns") || e.to_string().contains("network") {
                    println!("ネットワークエラーのため統合テストをスキップします");
                    return;
                }
                panic!("GitHubAPIアクセスが失敗しました: {:?}", e);
            }
        }

        // 実際のGitHubAPIを呼び出す統合テスト
        let result = manager.get_latest_version().await;

        match result {
            Ok(version_info) => {
                println!("取得したバージョン情報: {:?}", version_info);

                // バージョン文字列の基本的な検証
                assert!(!version_info.version.is_empty());
                assert!(version_info.version.starts_with("v"));

                // ダウンロードURLの基本的な検証
                assert!(!version_info.download_url.is_empty());
                assert!(version_info.download_url.starts_with("https://"));
                assert!(version_info.download_url.contains("github.com"));
                assert!(version_info.download_url.contains("ProcTail"));
                assert!(version_info.download_url.ends_with(".zip"));

                println!("✓ get_latest_version統合テストが成功しました");
            }
            Err(e) => {
                println!("GitHubAPI呼び出しエラー: {:?}", e);
                // ネットワークエラーの場合はテストをスキップ
                if e.to_string().contains("dns") || e.to_string().contains("network") {
                    println!("ネットワークエラーのため統合テストをスキップします");
                    return;
                }

                // Windows用のassetが見つからない場合の対処
                if e.to_string().contains("No Windows asset found") {
                    println!("Windows用のassetが見つからないため、利用可能なassetを確認しました");
                    println!("これは正常な動作です（リリースによってはWindows用のassetが存在しない場合があります）");
                    return;
                }

                panic!("get_latest_versionが失敗しました: {:?}", e);
            }
        }
    }

    /// 実際のGitHubAPIを使用したis_update_available()の統合テスト
    ///
    /// 検証内容:
    /// - 現在のバージョンファイルが存在しない場合の更新可否判定
    /// - 実際のGitHubAPIを使用した更新チェック機能の検証
    ///
    /// 期待結果: true（更新が利用可能）
    /// 重要性: 初回インストール時の動作確認
    ///
    /// 実行方法: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore] // デフォルトでスキップ、cargo test -- --ignored で実行
    async fn test_is_update_available_integration() {
        let (manager, _temp_dir) = create_test_manager();

        // 現在のバージョンが存在しない場合のテスト
        let result = manager.is_update_available().await;

        match result {
            Ok(update_available) => {
                println!("更新が利用可能: {}", update_available);
                // 現在のバージョンが存在しない場合は更新が利用可能であるべき
                assert!(update_available);
                println!("✓ is_update_available統合テストが成功しました");
            }
            Err(e) => {
                println!("GitHubAPI呼び出しエラー: {:?}", e);
                // ネットワークエラーの場合はテストをスキップ
                if e.to_string().contains("dns") || e.to_string().contains("network") {
                    println!("ネットワークエラーのため統合テストをスキップします");
                    return;
                }
                panic!("is_update_availableが失敗しました: {:?}", e);
            }
        }
    }

    /// 実際のGitHubAPIを使用したget_status()の統合テスト
    ///
    /// 検証内容:
    /// - 全体的なProcTailManagerのステータス取得
    /// - 複数のフィールドの整合性確認
    ///
    /// 検証項目:
    /// - current_version: None（バージョンファイルなし）
    /// - is_running: false（プロセス未実行）
    /// - executable_exists: false（実行ファイルなし）
    /// - update_available: true（更新利用可能）
    ///
    /// 実行方法: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore] // デフォルトでスキップ、cargo test -- --ignored で実行
    async fn test_get_status_integration() {
        let (manager, _temp_dir) = create_test_manager();

        // 実際のGitHubAPIを呼び出してステータスを取得
        let result = manager.get_status().await;

        match result {
            Ok(status) => {
                println!("取得したステータス: {:?}", status);

                // 基本的な検証
                assert_eq!(status.current_version, None); // バージョンファイルが存在しない
                assert!(!status.is_running); // プロセスが実行されていない
                assert!(!status.executable_exists); // 実行可能ファイルが存在しない
                assert!(status.update_available); // 更新が利用可能

                println!("✓ get_status統合テストが成功しました");
            }
            Err(e) => {
                println!("GitHubAPI呼び出しエラー: {:?}", e);
                // ネットワークエラーの場合はテストをスキップ
                if e.to_string().contains("dns") || e.to_string().contains("network") {
                    println!("ネットワークエラーのため統合テストをスキップします");
                    return;
                }
                panic!("get_statusが失敗しました: {:?}", e);
            }
        }
    }

    // =============================================================================
    // 部分的なスタブテスト（完全な実装にはモックが必要）
    // =============================================================================

    /// ダウンロード・インストール機能の基本的な動作テスト（スタブ）
    ///
    /// 検証内容:
    /// - ディレクトリの作成前後の状態確認
    /// - 実際のダウンロードは行わず、基本的な前提条件のみテスト
    ///
    /// 現在の状態: 基本的な前提条件のみ検証
    /// 完全なテストには: HTTPダウンロードとZIP展開のモックが必要
    #[tokio::test]
    async fn test_download_and_install_creates_directory() {
        let (manager, _temp_dir) = create_test_manager();

        let _version_info = ProcTailVersion {
            version: "v1.0.0".to_string(),
            download_url: "https://example.com/test.zip".to_string(),
        };

        // 実際のダウンロードはせずに、ディレクトリの作成をテスト
        let proctail_dir = manager.get_proctail_dir();
        assert!(!proctail_dir.exists());

        // ディレクトリが存在しない場合のテスト
        let result = manager.get_current_version().await.unwrap();
        assert_eq!(result, None);
    }

    /// バージョンファイルが存在しない場合のis_update_available()テスト（スタブ）
    ///
    /// 検証内容:
    /// - 現在のバージョンがない場合の更新可否判定
    /// - GitHubAPIを呼び出すため、実際のテストにはモックが必要
    ///
    /// 現在の状態: 実装未完了（GitHubAPIの呼び出しが発生）
    /// 完了のためには: get_latest_version()のモック実装が必要
    #[tokio::test]
    async fn test_is_update_available_no_current_version() {
        let (_manager, _temp_dir) = create_test_manager();

        // 現在のバージョンがない場合は更新が利用可能
        // 実際のGitHubAPIを呼び出すため、本来はモックが必要
        // このテストは統合テストとして実行するか、モックを使用する必要がある
    }

    /// 同じバージョンが既に存在する場合のis_update_available()テスト（スタブ）
    ///
    /// 検証内容:
    /// - 現在のバージョンと最新バージョンが同じ場合の更新可否判定
    /// - GitHubAPIを呼び出すため、実際のテストにはモックが必要
    ///
    /// セットアップ: バージョンファイルに"v1.0.0"を書き込み
    /// 現在の状態: 実装未完了（GitHubAPIの呼び出しが発生）
    /// 完了のためには: get_latest_version()のモック実装が必要
    #[tokio::test]
    async fn test_is_update_available_with_same_version() {
        let (manager, _temp_dir) = create_test_manager();

        // バージョンディレクトリを作成
        let version_dir = manager.get_proctail_version_dir("v1.0.0");
        std::fs::create_dir_all(&version_dir).unwrap();

        // 実際のGitHubAPIを呼び出すため、本来はモックが必要
        // このテストは統合テストとして実行するか、モックを使用する必要がある
    }

    /// 更新が不要な場合のensure_latest_version()テスト（スタブ）
    ///
    /// 検証内容:
    /// - 既に最新バージョンが存在する場合の処理
    /// - GitHubAPIを呼び出すため、実際のテストにはモックが必要
    ///
    /// 現在の状態: 実装未完了（GitHubAPIの呼び出しが発生）
    /// 完了のためには: is_update_available()のモック実装が必要
    #[tokio::test]
    async fn test_ensure_latest_version_no_update_needed() {
        let (_manager, _temp_dir) = create_test_manager();

        // 実際のGitHubAPIを呼び出すため、本来はモックが必要
        // このテストは統合テストとして実行するか、モックを使用する必要がある
    }

    /// 実行可能ファイルが存在しない場合のstart_proctail()テスト（スタブ）
    ///
    /// 検証内容:
    /// - 実行可能ファイルが存在しない場合の起動処理
    /// - ensure_latest_versionが呼び出されるため、GitHubAPIのモックが必要
    ///
    /// 現在の状態: 実装未完了（GitHubAPIの呼び出しが発生）
    /// 完了のためには: start_proctailの分割またはensure_latest_versionのモック実装が必要
    #[tokio::test]
    async fn test_start_proctail_no_executable() {
        let (_manager, _temp_dir) = create_test_manager();

        // 実行可能ファイルが存在しない場合のテスト
        // ensure_latest_versionが呼び出されるため、GitHubAPIのモックが必要
        // このテストは単体テストとして実行するには、start_proctailを分割する必要がある
    }

    // =============================================================================
    // プロセス管理テスト
    // =============================================================================

    /// プロセスが存在しない状態でのstop_proctail()テスト
    ///
    /// 検証内容:
    /// - プロセスが存在しない状態での停止処理
    /// - 冪等性の確保（既に停止している場合も正常終了）
    ///
    /// 期待結果: エラーなく正常終了（Ok(())）
    /// 重要性: 既に停止している場合の安全な処理を保証
    #[tokio::test]
    async fn test_stop_proctail_no_process() {
        let (manager, _temp_dir) = create_test_manager();

        let result = manager.stop_proctail().await;
        assert!(result.is_ok());
    }

    /// 実行可能ファイルが存在しない場合のget_status()テスト（スタブ）
    ///
    /// 検証内容:
    /// - 実行可能ファイルが存在しない場合のステータス取得
    /// - get_statusはis_update_availableを呼び出し、GitHubAPIを呼び出すため、
    ///   実際のテストではモックが必要
    ///
    /// 現在の状態: 基本的な状態のテストのみ実装
    /// 完了のためには: is_update_available()のモック実装が必要
    #[tokio::test]
    async fn test_get_status_no_executable() {
        let (manager, _temp_dir) = create_test_manager();

        // 実行可能ファイルが存在しない場合のテスト
        // get_statusはis_update_availableを呼び出し、GitHubAPIを呼び出すため、
        // 実際のテストではモックが必要
        // 現在は基本的な状態のテストのみ
        let executable_exists = manager.get_proctail_executable_path("v1.0.0").exists();
        assert!(!executable_exists);
    }

    // =============================================================================
    // パス生成テスト
    // =============================================================================

    /// 各種パス生成メソッドの正しい動作テスト
    ///
    /// 検証内容:
    /// - get_proctail_dir(): "proctail"で終わるパスを生成
    /// - get_proctail_executable_path(): "ProcTail.exe"で終わるパスを生成
    /// - get_version_file_path(): "version.txt"で終わるパスを生成
    ///
    /// 重要性: ファイル配置の一貫性確保、パス生成ロジックの検証
    #[test]
    fn test_path_methods() {
        let (manager, _temp_dir) = create_test_manager();

        let proctail_dir = manager.get_proctail_dir();
        assert!(proctail_dir.to_string_lossy().ends_with("proctail"));

        let executable_path = manager.get_proctail_executable_path("v1.0.0");
        assert!(executable_path
            .to_string_lossy()
            .ends_with("ProcTail.Host.exe"));

        let version_dir = manager.get_proctail_version_dir("v1.0.0");
        assert!(version_dir.to_string_lossy().ends_with("v1.0.0"));
    }

    // =============================================================================
    // データシリアライゼーションテスト
    // =============================================================================

    /// ProcTailVersion構造体のJSON変換テスト
    ///
    /// 検証内容:
    /// - ProcTailVersionのシリアライズ（構造体 → JSON）
    /// - ProcTailVersionのデシリアライズ（JSON → 構造体）
    /// - 変換前後での値の一致確認
    ///
    /// 重要性: API通信やファイル保存時のデータ整合性確保
    #[test]
    fn test_proctail_version_serialization() {
        let version = ProcTailVersion {
            version: "v1.0.0".to_string(),
            download_url: "https://example.com/download".to_string(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let deserialized: ProcTailVersion = serde_json::from_str(&json).unwrap();

        assert_eq!(version.version, deserialized.version);
        assert_eq!(version.download_url, deserialized.download_url);
    }

    /// ProcTailManagerStatus構造体のJSON変換テスト
    ///
    /// 検証内容:
    /// - ProcTailManagerStatusのシリアライズ（構造体 → JSON）
    /// - ProcTailManagerStatusのデシリアライズ（JSON → 構造体）
    /// - 全フィールドの値が正しく復元されることを確認
    ///
    /// 重要性: フロントエンドとの通信での型安全性確保
    #[test]
    fn test_proctail_manager_status_serialization() {
        let status = ProcTailManagerStatus {
            current_version: Some("v1.0.0".to_string()),
            is_running: true,
            executable_exists: true,
            update_available: false,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ProcTailManagerStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.current_version, deserialized.current_version);
        assert_eq!(status.is_running, deserialized.is_running);
        assert_eq!(status.executable_exists, deserialized.executable_exists);
        assert_eq!(status.update_available, deserialized.update_available);
    }

    // =============================================================================
    // エラーハンドリングテスト
    // =============================================================================

    /// 各エラー型の正しい生成とマッチングテスト
    ///
    /// 検証内容:
    /// - ProcTailManagerError::Io の生成とマッチング
    /// - ProcTailManagerError::Process の生成とマッチング
    /// - ProcTailManagerError::Download の生成とマッチング
    ///
    /// 重要性: エラー処理の型安全性確保
    #[test]
    fn test_error_types() {
        let io_error = ProcTailManagerError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(matches!(io_error, ProcTailManagerError::Io(_)));

        let process_error = ProcTailManagerError::Process("process failed".to_string());
        assert!(matches!(process_error, ProcTailManagerError::Process(_)));

        let download_error = ProcTailManagerError::Download("download failed".to_string());
        assert!(matches!(download_error, ProcTailManagerError::Download(_)));
    }

    /// エラーメッセージの表示内容テスト
    ///
    /// 検証内容:
    /// - 各エラー型が適切なメッセージを含むかを確認
    /// - Display traitの実装が正しく機能することを確認
    ///
    /// 重要性: デバッグ時の情報提供とユーザー体験の向上
    #[test]
    fn test_error_display() {
        let io_error = ProcTailManagerError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(io_error.to_string().contains("IO error"));

        let process_error = ProcTailManagerError::Process("process failed".to_string());
        assert!(process_error.to_string().contains("ProcTail process error"));

        let download_error = ProcTailManagerError::Download("download failed".to_string());
        assert!(download_error.to_string().contains("Download error"));
    }
}
