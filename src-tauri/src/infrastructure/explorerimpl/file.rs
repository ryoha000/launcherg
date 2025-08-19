use std::io::Write;
use std::sync::Arc;
use std::{fs, path::Path};

use base64::{engine::general_purpose, Engine as _};
use tauri::AppHandle;
use uuid::Uuid;

use crate::domain::{explorer::file::FileExplorer, file::File};
use crate::domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

use super::explorer::ExplorerImpl;

const MEMOS_ROOT_DIR: &str = "game-memos";
const SCREENSHOTS_ROOT_DIR: &str = "screenshots";

impl FileExplorer for ExplorerImpl<File> {
    fn save_base64_image(&self, path: &str, data: String) -> anyhow::Result<()> {
        let decoded_data = general_purpose::STANDARD_NO_PAD.decode(data)?;

        let mut file = std::fs::File::create(path)?;
        file.write_all(&decoded_data)?;
        Ok(())
    }
    fn get_save_image_path(&self, _handle: &Arc<AppHandle>, id: i32) -> anyhow::Result<String> {
        let resolver = DirsSavePathResolver::default();
        let dir = Path::new(&resolver.memos_dir()).join(id.to_string());
        fs::create_dir_all(&dir).unwrap();
        Ok(Path::new(&dir)
            .join(format!("{}.png", Uuid::new_v4().to_string()))
            .to_string_lossy()
            .to_string())
    }
    fn get_save_screenshot_path_by_name(
        &self,
        _handle: &Arc<AppHandle>,
        name: &str,
    ) -> anyhow::Result<String> {
        let resolver = DirsSavePathResolver::default();
        let dir = Path::new(&resolver.screenshots_dir()).to_path_buf();
        fs::create_dir_all(&dir).unwrap();
        let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
        Ok(Path::new(&dir)
            .join(format!("{}-{}.png", name, timestamp))
            .to_string_lossy()
            .to_string())
    }
    fn get_md_path(&self, _handle: &Arc<AppHandle>, id: i32) -> anyhow::Result<String> {
        let resolver = DirsSavePathResolver::default();
        let dir = Path::new(&resolver.memos_dir()).join(id.to_string());
        fs::create_dir_all(&dir).unwrap();
        Ok(Path::new(&dir)
            .join("untitled.md")
            .to_string_lossy()
            .to_string())
    }
}
