#![cfg(test)]
#![allow(non_snake_case)]

use std::collections::HashSet;
use std::sync::Arc;
use std::path::Path;

use tauri::Manager;

use crate::interface::module::Modules;
use crate::interface::command;
use crate::infrastructure::sqliterepository::driver::Db;
use crate::infrastructure::sqliterepository::sqliterepository::SqliteRepositoryManager;
use domain::repository::manager::RepositoryManager;
use domain::repository::RepositoriesExt;
use domain::repository::collection::CollectionRepository as _;
use domain::repository::explored_cache::ExploredCacheRepository as _;
use domain::service::save_path_resolver::SavePathResolver;

fn copy_file(src: &Path, dst: &Path) {
    let _ = std::fs::create_dir_all(dst.parent().unwrap());
    std::fs::copy(src, dst).expect("failed to copy db file");
}

// 環境変数でのルート切替は行わず、Db を直接渡して Modules を初期化する

fn build_app_with_plugins() -> tauri::App {
    tauri::Builder::default()
        .any_thread()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .expect("build tauri app")
}

async fn init_app_and_modules_with_db(db: Db) -> (tauri::App, tauri::AppHandle, Arc<Modules>) {
    let app = build_app_with_plugins();
    let handle = app.handle().clone();
    let modules = Arc::new(Modules::new(db, &handle).await);
    app.manage(modules.clone());
    (app, handle, modules)
}

async fn snapshot(db_path: &str) -> anyhow::Result<(Vec<(i32, i32)>, Vec<(i32, i32)>, Vec<String>, std::collections::HashSet<String>)> {
    let db = Db::from_path(db_path).await;
    let manager = Arc::new(SqliteRepositoryManager::new(db.pool_arc()));
    use std::collections::HashSet;
    let (egs_to_ce, ce_to_work, titles, explored): (Vec<(i32, i32)>, Vec<(i32, i32)>, Vec<String>, HashSet<String>) = manager.run(|repos| {
        Box::pin(async move {
            let mut coll = repos.collection();
            // すべての要素を取得しタイトルと EGS マッピングを拾う
            let all = coll.get_all_elements().await.unwrap_or_default();
            let titles = all.iter().map(|e| e.gamename.clone()).collect::<Vec<_>>();
            let mut egs_pairs: Vec<(i32, i32)> = Vec::new();
            let ce_ids: Vec<domain::Id<domain::collection::CollectionElement>> = all.iter().map(|e| e.id.clone()).collect();
            for e in all.iter() {
                if let Some(egs) = e.erogamescape.as_ref() {
                    egs_pairs.push((egs.erogamescape_id, e.id.value));
                }
            }
            // CE→Work の全体を解決
            let ce_work = coll.get_work_ids_by_collection_ids(&ce_ids).await.unwrap_or_default()
                .into_iter()
                .map(|(ce, wid)| (ce.value, wid.value))
                .collect::<Vec<_>>();
            let explored = repos.explored_cache().get_all().await.unwrap_or_default();
            Ok::<_, anyhow::Error>((egs_pairs, ce_work, titles, explored))
        })
    }).await?;
    Ok((egs_to_ce, ce_to_work, titles, explored))
}

fn write_dummy_exe(path: &Path) {
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    // 空ファイルで十分（存在のみで候補抽出される）
    let _ = std::fs::write(path, b"\x4D\x5A");
}

async fn pick_any_game_from_db(db_path: &str) -> anyhow::Result<(i32, String)> {
    // all_game_cache から 1 件取得
    let db = Db::from_path(db_path).await;
    let pool = db.pool_arc();
    let (id, name): (i32, String) = sqlx::query_as("SELECT id, gamename FROM all_game_caches LIMIT 1")
        .fetch_one(&*pool)
        .await?;
    Ok((id, name))
}

#[tokio::test]
async fn create_elements_in_pc_と_scan_start_で生成結果が等しい_単一exe() {
    // 前提: 実環境 DB を取得
    let real_root = domain::service::save_path_resolver::DirsSavePathResolver::default().root_dir();
    let real_db = Path::new(&real_root).join("launcherg_sqlite.db3");
    assert!(real_db.exists(), "事前に AllGameCache が初期化された DB が必要です: {:?}", real_db);

    // A/B のコピー
    let tmp_a = tempfile::TempDir::new().unwrap();
    let tmp_b = tempfile::TempDir::new().unwrap();
    let db_a = tmp_a.path().join("ryoha.moe").join("launcherg").join("launcherg_sqlite.db3");
    let db_b = tmp_b.path().join("ryoha.moe").join("launcherg").join("launcherg_sqlite.db3");
    copy_file(&real_db, &db_a);
    copy_file(&real_db, &db_b);

    // 任意のゲーム名を DB から選ぶ
    let (_egs_id_a, gamename) = pick_any_game_from_db(&db_a.to_string_lossy()).await.unwrap();

    // 入力ファイルを準備（GameName/GameName.exe）
    let roots_tmp = tempfile::TempDir::new().unwrap();
    let game_dir = roots_tmp.path().join(&gamename);
    let exe = game_dir.join(format!("{}.exe", &gamename));
    write_dummy_exe(&exe);

    // 事前スナップショット（差分比較用）
    let before_a = snapshot(&db_a.to_string_lossy()).await.unwrap();
    let before_b = snapshot(&db_b.to_string_lossy()).await.unwrap();

    // -------- A: create_elements_in_pc --------
    let db_a_loaded = Db::from_path(&db_a.to_string_lossy()).await;
    let (app_a, handle_a, _modules_a) = init_app_and_modules_with_db(db_a_loaded).await;
    let state_a: tauri::State<'_, Arc<Modules>> = app_a.state::<Arc<Modules>>();
    let _ = command::create_elements_in_pc(state_a, handle_a.clone(), vec![roots_tmp.path().to_string_lossy().to_string()], true).await.unwrap();
    let snap_a = snapshot(&db_a.to_string_lossy()).await.unwrap();

    // -------- B: scan_start --------
    let db_b_loaded = Db::from_path(&db_b.to_string_lossy()).await;
    let (app_b, _handle_b, _modules_b) = init_app_and_modules_with_db(db_b_loaded).await;
    let state_b: tauri::State<'_, Arc<Modules>> = app_b.state::<Arc<Modules>>();
    let _ = command::scan_start(state_b, vec![roots_tmp.path().to_string_lossy().to_string()], Some(true)).await.unwrap();
    let snap_b = snapshot(&db_b.to_string_lossy()).await.unwrap();

    // 比較（差分のみ）。画像/サイズや既存キャッシュの差は無視する
    let (egs_ce_a, _w1, _t1, explored_a) = snap_a;
    let (egs_ce_b, _w2, _t2, explored_b) = snap_b;
    let (egs_before_a, _wb1, _tb1, explored_before_a) = before_a;
    let (egs_before_b, _wb2, _tb2, explored_before_b) = before_b;

    let egs_set_after_a: std::collections::HashSet<_> = egs_ce_a.iter().map(|(egs, _)| *egs).collect();
    let egs_set_before_a: std::collections::HashSet<_> = egs_before_a.iter().map(|(egs, _)| *egs).collect();
    let added_egs_a: std::collections::HashSet<_> = egs_set_after_a.difference(&egs_set_before_a).cloned().collect();

    let egs_set_after_b: std::collections::HashSet<_> = egs_ce_b.iter().map(|(egs, _)| *egs).collect();
    let egs_set_before_b: std::collections::HashSet<_> = egs_before_b.iter().map(|(egs, _)| *egs).collect();
    let added_egs_b: std::collections::HashSet<_> = egs_set_after_b.difference(&egs_set_before_b).cloned().collect();

    assert_eq!(added_egs_a, added_egs_b, "追加された EGS が一致しません");

    let added_explored_a: std::collections::HashSet<_> = explored_a.difference(&explored_before_a).cloned().collect();
    let added_explored_b: std::collections::HashSet<_> = explored_b.difference(&explored_before_b).cloned().collect();
    assert_eq!(added_explored_a, added_explored_b, "追加された explored_cache が一致しません");
}


#[tokio::test]
#[ignore]
async fn create_elements_in_pc_と_scan_start_で生成結果が等しい_root_Gドライブ指定() {
    // 前提: 実環境 DB を取得
    let real_root = domain::service::save_path_resolver::DirsSavePathResolver::default().root_dir();
    let real_db = Path::new(&real_root).join("launcherg_sqlite.db3");
    assert!(real_db.exists(), "事前に AllGameCache が初期化された DB が必要です: {:?}", real_db);

    // A/B のコピー
    let tmp_a = tempfile::TempDir::new().unwrap();
    let tmp_b = tempfile::TempDir::new().unwrap();
    let db_a = tmp_a.path().join("ryoha.moe").join("launcherg").join("launcherg_sqlite.db3");
    let db_b = tmp_b.path().join("ryoha.moe").join("launcherg").join("launcherg_sqlite.db3");
    copy_file(&real_db, &db_a);
    copy_file(&real_db, &db_b);

    // -------- A: create_elements_in_pc --------
    let db_a_loaded = Db::from_path(&db_a.to_string_lossy()).await;
    let (app_a, handle_a, _modules_a) = init_app_and_modules_with_db(db_a_loaded).await;
    let state_a: tauri::State<'_, Arc<Modules>> = app_a.state::<Arc<Modules>>();
    let start_create_elements_in_pc = std::time::Instant::now();
    let _ = command::create_elements_in_pc(state_a, handle_a.clone(), vec!["G:\\game".to_string()], false).await;
    let processing_time_create_elements_in_pc = start_create_elements_in_pc.elapsed();
    let snap_a = snapshot(&db_a.to_string_lossy()).await.unwrap();

    // -------- B: scan_start --------
    // Db 直渡しで初期化するため環境変数の切り替えは不要
    // WorkPipelineUseCase を直接構築して実行し、フェーズ時間は PubSub 経由で収集する
    #[derive(Clone, Default)]
    struct TestPubSub { events: std::sync::Arc<std::sync::Mutex<Vec<(String, serde_json::Value)>>> }
    impl domain::pubsub::PubSubService for TestPubSub {
        fn notify<T: serde::Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error> {
            let val = serde_json::to_value(payload)?;
            self.events.lock().unwrap().push((event.to_string(), val));
            Ok(())
        }
    }

    use crate::infrastructure::sqliterepository::driver::Db as TestDb;
    use crate::infrastructure::sqliterepository::sqliterepository::SqliteRepositoryManager as TestRepoManager;
    use crate::infrastructure::local_file_system::LocalFileSystem;
    use crate::infrastructure::heuristic_metadata_extractor::HeuristicMetadataExtractor;
    use crate::infrastructure::heuristic_duplicate_resolver::HeuristicDuplicateResolver;
    use crate::infrastructure::windowsimpl::windows::Windows as InfraWindows;
    use domain::game_matcher::{Matcher as GameMatcherImpl, GameMatcher, normalize};
    use domain::all_game_cache::AllGameCacheOne as DomainAllGameCacheOne;
    use domain::repository::manager::RepositoryManager as _;

    use domain::repository::all_game_cache::AllGameCacheRepository as _;
    let test_db = TestDb::from_path(&db_b.to_string_lossy()).await;
    let test_repo_manager = std::sync::Arc::new(TestRepoManager::new(test_db.pool_arc()));
    // GameMatcher をDBから初期化
    let initial_cache = test_repo_manager
        .run(|repos| Box::pin(async move { repos.all_game_cache().get_all().await }))
        .await
        .unwrap_or_else(|_| vec![]);
    let normalized_cache: Vec<DomainAllGameCacheOne> = initial_cache
        .into_iter()
        .map(|g| DomainAllGameCacheOne::new(g.id, normalize(&g.gamename)))
        .collect();
    let game_matcher: std::sync::Arc<dyn GameMatcher + Send + Sync> = std::sync::Arc::new(GameMatcherImpl::with_default_config(normalized_cache));
    let extractor = std::sync::Arc::new(HeuristicMetadataExtractor::new(game_matcher));
    let fs = std::sync::Arc::new(LocalFileSystem::default());
    let dedup = std::sync::Arc::new(HeuristicDuplicateResolver);
    let pubsub = TestPubSub::default();

    let windows = std::sync::Arc::new(InfraWindows::new());
    let resolver = std::sync::Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default());
    let uc: crate::usecase::work_pipeline::WorkPipelineUseCase<_, _, _, _, _, _, _> =
        crate::usecase::work_pipeline::WorkPipelineUseCase::new(test_repo_manager, pubsub.clone(), fs, extractor, dedup, resolver, windows);

    let start_scan_start = std::time::Instant::now();
    let _ = uc.start(vec![std::path::PathBuf::from("G:\\game")], false).await;
    let processing_time_scan_start = start_scan_start.elapsed();
    let snap_b = snapshot(&db_b.to_string_lossy()).await.unwrap();

    // 比較（アイコン/サムネイルの物理ファイルや image_queue は比較対象外）
    let (egs_ce_a, _ce_work_a, _titles_a, explored_a) = snap_a;
    let (egs_ce_b, _ce_work_b, _titles_b, explored_b) = snap_b;

    let egs_a = egs_ce_a.iter().map(|(egs, _)| egs).collect::<HashSet<_>>();
    let egs_b = egs_ce_b.iter().map(|(egs, _)| egs).collect::<HashSet<_>>();
    // assert_eq!(egs_a, egs_b, "EGS 集合が一致しません");

    // assert_eq!(explored_a, explored_b, "explored_cache が一致しません");

    // 受信したフェーズ時間を標準出力へ出す（目視確認用）
    let received_timings: Vec<(String, i64)> = pubsub
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| if k == "scanPhaseTiming" {
            let phase = v.get("phase").and_then(|s| s.as_str()).unwrap_or("").to_string();
            let dur = v.get("duration_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            Some((phase, dur))
        } else { None })
        .collect();
    assert!(!received_timings.is_empty(), "scanPhaseTiming が届いていない");
    for (phase, ms) in received_timings.into_iter() {
        println!("scanPhaseTiming: phase={} duration={}ms", phase, ms);
    }

    // パイプライン統計を標準出力へ出す
    let received_stats: Vec<(i64, i64, i64, i64, i64, i64, f64, f64)> = pubsub
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| if k == "scanPipelineStats" {
            let enumerated = v.get("enumerated_count").and_then(|n| n.as_i64()).unwrap_or(0);
            let processed = v.get("processed_count").and_then(|n| n.as_i64()).unwrap_or(0);
            let walking_ms = v.get("walking_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            let enriching_ms = v.get("enriching_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            let backlog_ms = v.get("backlog_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            let producer_block_ms = v.get("producer_block_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            let producer_rate_per_s = v.get("producer_rate_per_s").and_then(|n| n.as_f64()).unwrap_or(0.0);
            let consumer_rate_per_s = v.get("consumer_rate_per_s").and_then(|n| n.as_f64()).unwrap_or(0.0);
            Some((enumerated, processed, walking_ms, enriching_ms, backlog_ms, producer_block_ms, producer_rate_per_s, consumer_rate_per_s))
        } else { None })
        .collect();
    for (enum_cnt, proc_cnt, w_ms, e_ms, back_ms, block_ms, pr, cr) in received_stats.into_iter() {
        println!(
            "scanPipelineStats: enumerated={} processed={} walking_ms={} enriching_ms={} backlog_ms={} producer_block_ms={} producer_rate/s={:.2} consumer_rate/s={:.2}",
            enum_cnt, proc_cnt, w_ms, e_ms, back_ms, block_ms, pr, cr
        );
    }

    println!("processing_time_scan_start: {:?}", processing_time_scan_start);
    println!("processing_time_create_elements_in_pc: {:?}", processing_time_create_elements_in_pc);

    // assert!(processing_time_scan_start < processing_time_create_elements_in_pc, "scan_start の処理時間が create_elements_in_pc の処理時間より長い. {:?} > {:?}", processing_time_scan_start, processing_time_create_elements_in_pc);
}


