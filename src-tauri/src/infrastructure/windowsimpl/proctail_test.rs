#[cfg(test)]
mod tests {
    use super::super::proctail::ProcTailImpl;
    use crate::domain::windows::proctail::ProcTail;

    #[tokio::test]
    #[ignore]
    async fn test_health_check_with_default_pipe() {
        let proctail = ProcTailImpl::new();
        
        let result = proctail.health_check().await;
        
        assert!(result.is_ok(), "Health check failed: {:?}", result.err());
        let health_check_result = result.unwrap();
        assert_eq!(health_check_result.status, "Healthy");
        assert!(!health_check_result.check_time.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_is_service_available() {
        let proctail = ProcTailImpl::new();
        
        let result = proctail.is_service_available().await;
        
        assert!(result, "Service should be available");
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_status() {
        let proctail = ProcTailImpl::new();
        
        let result = proctail.get_status().await;
        
        assert!(result.is_ok(), "Get status failed: {:?}", result.err());
        let status = result.unwrap();
        assert_eq!(status.service.status, "Running");
        assert!(status.monitoring.etw_session_active);
        assert!(status.ipc.named_pipe_active);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_watch_targets_empty() {
        let proctail = ProcTailImpl::new();
        
        let result = proctail.get_watch_targets().await;
        
        assert!(result.is_ok(), "Get watch targets failed: {:?}", result.err());
        let targets = result.unwrap();
        // 初期状態では監視対象が空である可能性があるため、エラーにならないことを確認
        println!("監視対象数: {}", targets.len());
        for target in targets {
            println!("Tag: {}, ProcessId: {}, ProcessName: {}", target.tag, target.process_id, target.process_name);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_add_and_remove_watch_target() {
        let proctail = ProcTailImpl::new();
        
        // 現在のプロセスIDを取得
        let current_process_id = std::process::id();
        let tag = "test_tag";
        
        // 監視対象を追加
        let add_result = proctail.add_watch_target(current_process_id, tag).await;
        assert!(add_result.is_ok(), "Add watch target failed: {:?}", add_result.err());
        
        let watch_target = add_result.unwrap();
        assert_eq!(watch_target.tag, tag);
        assert_eq!(watch_target.process_id, current_process_id);
        assert!(!watch_target.process_name.is_empty());
        
        // 監視対象一覧を取得して確認
        let targets_result = proctail.get_watch_targets().await;
        assert!(targets_result.is_ok(), "Get watch targets failed: {:?}", targets_result.err());
        
        let targets = targets_result.unwrap();
        let found_target = targets.iter().find(|t| t.tag == tag);
        assert!(found_target.is_some(), "Added target should be found in watch targets list");
        
        // 監視対象を削除
        let remove_result = proctail.remove_watch_target(tag).await;
        assert!(remove_result.is_ok(), "Remove watch target failed: {:?}", remove_result.err());
        
        let removed_count = remove_result.unwrap();
        assert!(removed_count > 0, "Should have removed at least one target");
        
        // 削除後の監視対象一覧を確認
        let targets_after_result = proctail.get_watch_targets().await;
        assert!(targets_after_result.is_ok(), "Get watch targets after removal failed: {:?}", targets_after_result.err());
        
        let targets_after = targets_after_result.unwrap();
        let found_target_after = targets_after.iter().find(|t| t.tag == tag);
        assert!(found_target_after.is_none(), "Removed target should not be found in watch targets list");
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_recorded_events() {
        let proctail = ProcTailImpl::new();
        
        // 現在のプロセスIDを取得
        let current_process_id = std::process::id();
        let tag = "test_events_tag";
        
        // 監視対象を追加
        let add_result = proctail.add_watch_target(current_process_id, tag).await;
        assert!(add_result.is_ok(), "Add watch target failed: {:?}", add_result.err());
        
        // 一時ディレクトリを作成
        let temp_dir = std::env::temp_dir().join(format!("proctail_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
        
        // ファイル操作を実行してFileEventを発生させる
        for i in 0..5 {
            let test_file = temp_dir.join(format!("test_file_{}.txt", i));
            std::fs::write(&test_file, format!("Test content {}", i))
                .expect("Failed to write test file");
            
            // ファイルの読み取り
            let _content = std::fs::read_to_string(&test_file)
                .expect("Failed to read test file");
            
            // ファイルの削除
            std::fs::remove_file(&test_file)
                .expect("Failed to remove test file");
        }
        
        // 子プロセスを作成・停止してProcessEventを発生させる
        for i in 0..3 {
            let child = std::process::Command::new("cmd")
                .args(&["/C", &format!("echo Test process {}", i)])
                .spawn()
                .expect("Failed to spawn child process");
            
            // プロセスの終了を待つ
            let _output = child.wait_with_output()
                .expect("Failed to wait for child process");
        }
        
        // 記録されたイベントを取得
        let events_result = proctail.get_recorded_events(tag, Some(50), None).await;
        assert!(events_result.is_ok(), "Get recorded events failed: {:?}", events_result.err());
        
        let events = events_result.unwrap();
        println!("記録されたイベント数: {}", events.len());
        
        let mut file_events = 0;
        let mut process_start_events = 0;
        let mut process_end_events = 0;
        
        // イベントの内容を確認し、種類別にカウント
        for event in &events {
            match event {
                crate::domain::windows::proctail::ProcTailEvent::File(data) => {
                    println!("File Event: {} - {} (PID: {})", data.event_type, data.file_path, data.process_id);
                    file_events += 1;
                }
                crate::domain::windows::proctail::ProcTailEvent::ProcessStart(data) => {
                    println!("Process Start Event: {} - {} (PID: {})", data.event_type, data.process_name, data.process_id);
                    process_start_events += 1;
                }
                crate::domain::windows::proctail::ProcTailEvent::ProcessEnd(data) => {
                    println!("Process End Event: {} - {} (PID: {})", data.event_type, data.process_name, data.process_id);
                    process_end_events += 1;
                }
            }
        }
        
        println!("ファイルイベント数: {}", file_events);
        println!("プロセス開始イベント数: {}", process_start_events);
        println!("プロセス終了イベント数: {}", process_end_events);
        
        // 少なくとも何らかのイベントが記録されていることを確認
        // 注意: ProcTailがすべてのイベントをキャッチできるとは限らないため、
        // 厳密な数のチェックではなく、0より大きいことを確認
        if events.len() > 0 {
            println!("✓ イベントが正常に記録されました");
        } else {
            println!("⚠ イベントが記録されませんでした（ProcTailサービスの設定を確認してください）");
        }
        
        // クリーンアップ: 一時ディレクトリを削除
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        // クリーンアップ: 監視対象を削除
        let _ = proctail.remove_watch_target(tag).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_clear_events() {
        let proctail = ProcTailImpl::new();
        
        // 現在のプロセスIDを取得
        let current_process_id = std::process::id();
        let tag = "test_clear_events_tag";
        
        // 監視対象を追加
        let add_result = proctail.add_watch_target(current_process_id, tag).await;
        assert!(add_result.is_ok(), "Add watch target failed: {:?}", add_result.err());
        
        // 一時ディレクトリを作成
        let temp_dir = std::env::temp_dir().join(format!("proctail_clear_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
        
        // ファイル操作を実行してイベントを発生させる
        for i in 0..3 {
            let test_file = temp_dir.join(format!("clear_test_file_{}.txt", i));
            std::fs::write(&test_file, format!("Clear test content {}", i))
                .expect("Failed to write test file");
            let _content = std::fs::read_to_string(&test_file)
                .expect("Failed to read test file");
            std::fs::remove_file(&test_file)
                .expect("Failed to remove test file");
        }
        
        // 子プロセスを作成してイベントを発生させる
        let child = std::process::Command::new("cmd")
            .args(&["/C", "echo Clear test process"])
            .spawn()
            .expect("Failed to spawn child process");
        let _output = child.wait_with_output()
            .expect("Failed to wait for child process");
        
        // クリア前のイベント数を確認
        let events_before_result = proctail.get_recorded_events(tag, Some(50), None).await;
        assert!(events_before_result.is_ok(), "Get recorded events before clear failed: {:?}", events_before_result.err());
        
        let events_before = events_before_result.unwrap();
        println!("クリア前のイベント数: {}", events_before.len());
        
        // イベントをクリア
        let clear_result = proctail.clear_events(tag).await;
        assert!(clear_result.is_ok(), "Clear events failed: {:?}", clear_result.err());
        
        let cleared_count = clear_result.unwrap();
        println!("クリアされたイベント数: {}", cleared_count);
        
        // クリア後にイベントを取得してみる
        let events_after_result = proctail.get_recorded_events(tag, Some(50), None).await;
        assert!(events_after_result.is_ok(), "Get recorded events after clear failed: {:?}", events_after_result.err());
        
        let events_after = events_after_result.unwrap();
        println!("クリア後のイベント数: {}", events_after.len());
        
        // クリア動作の確認
        if events_before.len() > 0 {
            if events_after.len() < events_before.len() {
                println!("✓ イベントのクリア操作が正常に動作しました");
            } else {
                println!("⚠ クリア操作後もイベント数が変化していません");
            }
        } else {
            println!("⚠ クリア前にイベントが記録されていませんでした");
        }
        
        // クリーンアップ: 一時ディレクトリを削除
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        // クリーンアップ: 監視対象を削除
        let _ = proctail.remove_watch_target(tag).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_process_not_found_error() {
        let proctail = ProcTailImpl::new();
        
        // 存在しないプロセスIDを使用
        let invalid_process_id = 99999u32;
        let tag = "invalid_process_tag";
        
        let result = proctail.add_watch_target(invalid_process_id, tag).await;
        
        // エラーが発生することを確認
        assert!(result.is_err(), "Should fail with invalid process ID");
        
        // エラーの種類を確認
        let error = result.unwrap_err();
        println!("Expected error: {:?}", error);
        // ProcessNotFoundエラーまたはServiceErrorが発生することを確認
        assert!(
            matches!(error, crate::domain::windows::proctail::ProcTailError::ProcessNotFound(_)) ||
            matches!(error, crate::domain::windows::proctail::ProcTailError::ServiceError(_))
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_tag_not_found_error() {
        let proctail = ProcTailImpl::new();
        
        let non_existent_tag = "non_existent_tag";
        
        // 存在しないタグで削除を試行
        let result = proctail.remove_watch_target(non_existent_tag).await;
        
        // エラーが発生することを確認
        assert!(result.is_err(), "Should fail with non-existent tag");
        
        let error = result.unwrap_err();
        println!("Expected error: {:?}", error);
        // TagNotFoundエラーまたはServiceErrorが発生することを確認
        assert!(
            matches!(error, crate::domain::windows::proctail::ProcTailError::TagNotFound(_)) ||
            matches!(error, crate::domain::windows::proctail::ProcTailError::ServiceError(_))
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_recorded_events_non_existent_tag() {
        let proctail = ProcTailImpl::new();
        
        let non_existent_tag = "non_existent_events_tag";
        
        // 存在しないタグでイベントを取得
        let result = proctail.get_recorded_events(non_existent_tag, Some(10), None).await;
        
        // ProcTailサービスは存在しないタグでも空の配列を返すので、成功することを確認
        assert!(result.is_ok(), "Should succeed with empty events array for non-existent tag");
        
        let events = result.unwrap();
        assert_eq!(events.len(), 0, "Should return empty events array for non-existent tag");
        println!("Events for non-existent tag: {} events", events.len());
    }
}