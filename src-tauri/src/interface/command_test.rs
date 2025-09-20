#![cfg(test)]
#![allow(non_snake_case)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use tauri::Manager;

use crate::infrastructure::sqliterepository::driver::Db;
use crate::infrastructure::sqliterepository::sqliterepository::SqliteRepositoryManager;
use crate::interface::command;
use crate::interface::module::Modules;
use domain::repository::collection::CollectionRepository as _;
use domain::repository::explored_cache::ExploredCacheRepository as _;
use domain::repository::manager::RepositoryManager;
use domain::repository::RepositoriesExt;
use domain::service::save_path_resolver::SavePathResolver;

fn copy_file(src: &Path, dst: &Path) {
    let _ = std::fs::create_dir_all(dst.parent().unwrap());
    std::fs::copy(src, dst).expect("failed to copy db file");
}

// 環境変数でのルート切替は行わず、Db を直接渡して Modules を初期化する

fn build_app_with_plugins() -> tauri::App {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .expect("resolve current exe dir");
    let dst = exe_dir.join("extract-icon.exe");
    const BYTES: &[u8] = include_bytes!("../../bin/extract-icon-x86_64-pc-windows-msvc.exe");
    std::fs::write(&dst, BYTES).expect("write sidecar");
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

async fn snapshot(
    db_path: &str,
) -> anyhow::Result<(
    Vec<(i32, i32)>,
    Vec<(i32, i32)>,
    Vec<String>,
    std::collections::HashSet<String>,
)> {
    let db = Db::from_path(db_path).await;
    let manager = Arc::new(SqliteRepositoryManager::new(db.pool_arc()));
    use std::collections::HashSet;
    let (egs_to_ce, ce_to_work, titles, explored): (
        Vec<(i32, i32)>,
        Vec<(i32, i32)>,
        Vec<String>,
        HashSet<String>,
    ) = manager
        .run(|repos| {
            Box::pin(async move {
                let mut coll = repos.collection();
                // すべての要素を取得しタイトルと EGS マッピングを拾う
                let all = coll.get_all_elements().await.unwrap_or_default();
                let titles = all.iter().map(|e| e.gamename.clone()).collect::<Vec<_>>();
                let mut egs_pairs: Vec<(i32, i32)> = Vec::new();
                let ce_ids: Vec<domain::Id<domain::collection::CollectionElement>> =
                    all.iter().map(|e| e.id.clone()).collect();
                for e in all.iter() {
                    if let Some(egs) = e.erogamescape.as_ref() {
                        egs_pairs.push((egs.erogamescape_id, e.id.value));
                    }
                }
                // CE→Work の全体を解決
                let ce_work = coll
                    .get_work_ids_by_collection_ids(&ce_ids)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(ce, wid)| (ce.value, wid.value))
                    .collect::<Vec<_>>();
                let explored = repos.explored_cache().get_all().await.unwrap_or_default();
                Ok::<_, anyhow::Error>((egs_pairs, ce_work, titles, explored))
            })
        })
        .await?;
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
    let (id, name): (i32, String) =
        sqlx::query_as("SELECT id, gamename FROM all_game_caches LIMIT 1")
            .fetch_one(&*pool)
            .await?;
    Ok((id, name))
}

#[tokio::test]
async fn create_elements_in_pc_と_scan_start_で生成結果が等しい_単一exe() {
    // 前提: 実環境 DB を取得
    let real_root = domain::service::save_path_resolver::DirsSavePathResolver::default().root_dir();
    let real_db = Path::new(&real_root).join("launcherg_sqlite.db3");
    assert!(
        real_db.exists(),
        "事前に AllGameCache が初期化された DB が必要です: {:?}",
        real_db
    );

    // A/B のコピー
    let tmp_a = tempfile::TempDir::new().unwrap();
    let tmp_b = tempfile::TempDir::new().unwrap();
    let db_a = tmp_a
        .path()
        .join("ryoha.moe")
        .join("launcherg")
        .join("launcherg_sqlite.db3");
    let db_b = tmp_b
        .path()
        .join("ryoha.moe")
        .join("launcherg")
        .join("launcherg_sqlite.db3");
    copy_file(&real_db, &db_a);
    copy_file(&real_db, &db_b);

    // 任意のゲーム名を DB から選ぶ
    let (_egs_id_a, gamename) = pick_any_game_from_db(&db_a.to_string_lossy())
        .await
        .unwrap();

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
    let _ = command::create_elements_in_pc(
        state_a,
        handle_a.clone(),
        vec![roots_tmp.path().to_string_lossy().to_string()],
        true,
    )
    .await
    .unwrap();
    let snap_a = snapshot(&db_a.to_string_lossy()).await.unwrap();

    // -------- B: scan_start --------
    let db_b_loaded = Db::from_path(&db_b.to_string_lossy()).await;
    let (app_b, _handle_b, _modules_b) = init_app_and_modules_with_db(db_b_loaded).await;
    let state_b: tauri::State<'_, Arc<Modules>> = app_b.state::<Arc<Modules>>();
    let _ = command::scan_start(
        state_b,
        vec![roots_tmp.path().to_string_lossy().to_string()],
        Some(true),
    )
    .await
    .unwrap();
    let snap_b = snapshot(&db_b.to_string_lossy()).await.unwrap();

    // 比較（差分のみ）。画像/サイズや既存キャッシュの差は無視する
    let (egs_ce_a, _w1, _t1, explored_a) = snap_a;
    let (egs_ce_b, _w2, _t2, explored_b) = snap_b;
    let (egs_before_a, _wb1, _tb1, explored_before_a) = before_a;
    let (egs_before_b, _wb2, _tb2, explored_before_b) = before_b;

    let egs_set_after_a: std::collections::HashSet<_> =
        egs_ce_a.iter().map(|(egs, _)| *egs).collect();
    let egs_set_before_a: std::collections::HashSet<_> =
        egs_before_a.iter().map(|(egs, _)| *egs).collect();
    let added_egs_a: std::collections::HashSet<_> = egs_set_after_a
        .difference(&egs_set_before_a)
        .cloned()
        .collect();

    let egs_set_after_b: std::collections::HashSet<_> =
        egs_ce_b.iter().map(|(egs, _)| *egs).collect();
    let egs_set_before_b: std::collections::HashSet<_> =
        egs_before_b.iter().map(|(egs, _)| *egs).collect();
    let added_egs_b: std::collections::HashSet<_> = egs_set_after_b
        .difference(&egs_set_before_b)
        .cloned()
        .collect();

    assert_eq!(added_egs_a, added_egs_b, "追加された EGS が一致しません");

    let added_explored_a: std::collections::HashSet<_> =
        explored_a.difference(&explored_before_a).cloned().collect();
    let added_explored_b: std::collections::HashSet<_> =
        explored_b.difference(&explored_before_b).cloned().collect();
    assert_eq!(
        added_explored_a, added_explored_b,
        "追加された explored_cache が一致しません"
    );
}

#[tokio::test]
#[ignore]
async fn create_elements_in_pc_と_scan_start_で生成結果が等しい_root_Gドライブ指定() {
    // 前提: 実環境 DB を取得
    let real_root = domain::service::save_path_resolver::DirsSavePathResolver::default().root_dir();
    let real_db = Path::new(&real_root).join("launcherg_sqlite.db3");
    assert!(
        real_db.exists(),
        "事前に AllGameCache が初期化された DB が必要です: {:?}",
        real_db
    );

    // A/B のコピー
    let tmp_a = tempfile::TempDir::new().unwrap();
    let tmp_b = tempfile::TempDir::new().unwrap();
    let db_a = tmp_a
        .path()
        .join("ryoha.moe")
        .join("launcherg")
        .join("launcherg_sqlite.db3");
    let db_b = tmp_b
        .path()
        .join("ryoha.moe")
        .join("launcherg")
        .join("launcherg_sqlite.db3");
    copy_file(&real_db, &db_a);
    copy_file(&real_db, &db_b);

    // -------- A: create_elements_in_pc --------
    let db_a_loaded = Db::from_path(&db_a.to_string_lossy()).await;
    let (app_a, handle_a, _modules_a) = init_app_and_modules_with_db(db_a_loaded).await;
    let state_a: tauri::State<'_, Arc<Modules>> = app_a.state::<Arc<Modules>>();
    let start_create_elements_in_pc = std::time::Instant::now();
    let _ = command::create_elements_in_pc(
        state_a,
        handle_a.clone(),
        vec!["G:\\game".to_string()],
        false,
    )
    .await;
    let processing_time_create_elements_in_pc = start_create_elements_in_pc.elapsed();
    let snap_a = snapshot(&db_a.to_string_lossy()).await.unwrap();

    // -------- B: scan_start --------

    // -------- B: scan_start --------
    let db_b_loaded = Db::from_path(&db_b.to_string_lossy()).await;
    let (app_b, _handle_b, _modules_b) = init_app_and_modules_with_db(db_b_loaded).await;
    let state_b: tauri::State<'_, Arc<Modules>> = app_b.state::<Arc<Modules>>();
    let start_scan_start = std::time::Instant::now();
    let _ = command::scan_start(state_b, vec!["G:\\game".to_string()], Some(false))
        .await
        .unwrap();
    let processing_time_scan_start = start_scan_start.elapsed();
    let snap_b = snapshot(&db_b.to_string_lossy()).await.unwrap();

    // 比較（アイコン/サムネイルの物理ファイルや image_queue は比較対象外）
    let (egs_ce_a, _ce_work_a, _titles_a, explored_a) = snap_a;
    let (egs_ce_b, _ce_work_b, _titles_b, explored_b) = snap_b;

    let egs_a = egs_ce_a.iter().map(|(egs, _)| egs).collect::<HashSet<_>>();
    let egs_b = egs_ce_b.iter().map(|(egs, _)| egs).collect::<HashSet<_>>();
    assert_eq!(egs_a, egs_b, "EGS 集合が一致しません");

    assert_eq!(explored_a, explored_b, "explored_cache が一致しません");

    println!(
        "processing_time_scan_start: {:?}",
        processing_time_scan_start
    );
    println!(
        "processing_time_create_elements_in_pc: {:?}",
        processing_time_create_elements_in_pc
    );

    assert!(
        processing_time_scan_start < processing_time_create_elements_in_pc,
        "scan_start の処理時間が create_elements_in_pc の処理時間より長い. {:?} < {:?}",
        processing_time_scan_start,
        processing_time_create_elements_in_pc
    );
}
