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
        .invoke_handler(tauri::generate_handler![
            command::get_all_collections,
            command::get_collection_elements,
            command::add_collection_elements_in_pc,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
