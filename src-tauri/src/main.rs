// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::Arc;

use infrastructure::util::get_save_root_abs_dir_with_ptr_handle;
use interface::{command, module::Modules};
use tauri::{async_runtime::block_on, Manager};
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // folder の中身を移動して folder を削除する
            // C:\Users\ryoha\AppData\Roaming\launcherg -> C:\Users\ryoha\AppData\Roaming\ryoha.moe\launcherg

            let dst_dir = get_save_root_abs_dir_with_ptr_handle(app.handle());
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

            let modules = Arc::new(block_on(Modules::new(app.handle())));
            app.manage(modules);

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
            command::create_elements_in_pc,
            command::get_nearest_key_and_distance,
            command::upload_image,
            command::upsert_collection_element,
            command::update_collection_element_icon,
            command::get_default_import_dirs,
            command::play_game,
            command::get_play_time_minutes,
            command::get_collection_element,
            command::delete_collection_element,
            command::get_not_registered_detail_element_ids,
            command::create_element_details,
            command::get_all_elements,
            command::update_element_like,
            command::open_folder,
            command::get_all_game_cache_last_updated,
            command::update_all_game_cache,
            command::get_game_candidates,
            command::get_exe_path_by_lnk,
            command::get_game_cache_by_id,
            command::save_screenshot_by_pid,
            command::update_collection_element_thumbnails,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
