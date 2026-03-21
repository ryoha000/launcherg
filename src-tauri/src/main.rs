// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod interface;
pub mod domain {
    pub use ::domain::*;
}
pub mod infrastructure {
    pub use ::infrastructure::*;
}
pub mod usecase {
    pub use ::usecase::*;
}

use std::sync::Arc;

use ::infrastructure::sqliterepository::driver::Db;
use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};
use interface::{
    commands,
    module::{Modules, ModulesExt},
};
use tauri::{async_runtime::block_on, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            log::info!(
                "single instance triggered by deep link or duplicate launch: args={args:?}, cwd={cwd:?}"
            );

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // folder の中身を移動して folder を削除する
            // C:\Users\ryoha\AppData\Roaming\launcherg -> C:\Users\ryoha\AppData\Roaming\ryoha.moe\launcherg

            let dst_dir = DirsSavePathResolver::default().root_dir();
            let src_dir = std::path::Path::new(&dst_dir)
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("launcherg");
            println!("src_dir: {:?}, dst_dir: {:?}", src_dir, dst_dir);
            if src_dir.exists() {
                let dst_dir = std::path::Path::new(&dst_dir);
                std::fs::create_dir_all(&dst_dir).unwrap();
                for entry in std::fs::read_dir(&src_dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let file_name = path.file_name().unwrap();
                    let dst_path = dst_dir.join(file_name);
                    println!("rename {:?} -> {:?}", path, dst_path);
                    std::fs::rename(path, dst_path).unwrap();
                }
                std::fs::remove_dir_all(src_dir).unwrap();
            }

            let db = block_on(Db::new(&app.handle()));
            let modules = Arc::new(block_on(Modules::new(db, &app.handle())));
            app.manage(modules.clone());

            if let Err(err) =
                infrastructure::app_signal_router::interprocess::listener::spawn_listener(Arc::new(
                    modules.pubsub().clone(),
                ))
            {
                log::error!("failed to start app signal listener: {err}");
            }

            #[cfg(desktop)]
            {
                if let Err(err) = app.deep_link().register_all() {
                    log::error!("failed to register deep link schemes: {err}");
                }
            }

            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::utils::get_nearest_key_and_distance,
            commands::images::upload_image,
            commands::utils::get_default_import_dirs,
            commands::scan::scan_start,
            commands::utils::get_play_time_minutes,
            commands::erogamescape::upsert_erogamescape_information,
            commands::erogamescape::get_not_registered_erogamescape_information_ids,
            commands::works::update_work_like,
            commands::utils::open_folder,
            commands::all_game_cache::get_all_game_cache_last_updated,
            commands::all_game_cache::update_all_game_cache,
            commands::matcher::get_game_candidates,
            commands::utils::get_exe_path_by_lnk,
            commands::all_game_cache::get_game_cache_by_id,
            commands::images::save_screenshot_by_pid,
            commands::storage_paths::get_storage_settings,
            commands::storage_paths::set_storage_settings,
            commands::process_proctail::proctail_add_watch_target,
            commands::process_proctail::proctail_remove_watch_target,
            commands::process_proctail::proctail_get_watch_targets,
            commands::process_proctail::proctail_get_recorded_events,
            commands::process_proctail::proctail_clear_events,
            commands::process_proctail::proctail_get_status,
            commands::process_proctail::proctail_health_check,
            commands::process_proctail::proctail_is_service_available,
            commands::process_manager::proctail_manager_get_status,
            commands::process_manager::proctail_manager_get_latest_version,
            commands::process_manager::proctail_manager_is_update_available,
            commands::process_manager::proctail_manager_download_and_install,
            commands::process_manager::proctail_manager_start,
            commands::process_manager::proctail_manager_stop,
            commands::process_manager::proctail_manager_is_running,
            commands::utils::open_url,
            commands::matcher::get_game_candidates_by_name,
            commands::notification::show_os_notification,
            commands::extension::get_sync_status,
            commands::extension::set_extension_config,
            commands::extension::generate_extension_package,
            commands::extension::setup_native_messaging_host,
            commands::extension::get_extension_package_info,
            commands::extension::copy_extension_for_development,
            commands::extension::get_dev_extension_info,
            commands::extension::check_registry_keys,
            commands::extension::remove_registry_keys,
            commands::native_host_logs::get_native_host_logs,
            commands::work_details::get_work_details_all,
            commands::work_details::get_work_details_by_work_id,
            commands::works::backfill_thumbnail_sizes,
            commands::works::list_work_lnks,
            commands::works::get_work_paths,
            commands::works::launch_work,
            commands::works::delete_work,
            commands::works::register_work_from_path,
            commands::works::process_pending_exe_links,
            commands::image_queue::get_image_save_queue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
