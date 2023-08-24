// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod infrastructure;
mod interface;
mod usecase;

use std::sync::Arc;

use interface::{command, module::Modules};
use tauri::{async_runtime::block_on, Manager};

fn main() {
    let modules = Arc::new(block_on(Modules::new()));

    tauri::Builder::default()
        .setup(|app| {
            app.manage(modules);

            Ok(())
        })
        .plugin(tauri_plugin_clipboard::init())
        .invoke_handler(tauri::generate_handler![
            command::create_elements_in_pc,
            command::get_nearest_key_and_distance,
            command::upload_image,
            command::upsert_collection_element,
            command::update_collection_element_icon,
            command::remove_elements_from_collection,
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
            command::get_game_candidates
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
