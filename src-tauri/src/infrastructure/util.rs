use std::{fs, path::Path};

use tauri::{api::path::app_config_dir, Config};

const ROOT_DIR: &str = "launcherg";

pub fn get_save_root_abs_dir() -> String {
    let root = app_config_dir(&Config::default());
    match root {
        Some(root) => {
            let path = &root.join(Path::new(ROOT_DIR));
            fs::create_dir_all(path).unwrap();
            return path.to_string_lossy().to_string();
        }
        None => {
            fs::create_dir_all(ROOT_DIR).unwrap();
            return fs::canonicalize(ROOT_DIR)
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
    }
}
