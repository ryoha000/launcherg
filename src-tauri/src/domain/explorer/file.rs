use std::sync::Arc;

use async_trait::async_trait;
use tauri::AppHandle;

#[async_trait]
pub trait FileExplorer {
    fn save_base64_image(&self, path: &str, data: String) -> anyhow::Result<()>;
    fn get_save_image_path(&self, handle: &Arc<AppHandle>, id: i32) -> anyhow::Result<String>;
    fn get_save_screenshot_path_by_name(
        &self,
        handle: &Arc<AppHandle>,
        name: &str,
    ) -> anyhow::Result<String>;
    fn get_md_path(&self, handle: &Arc<AppHandle>, id: i32) -> anyhow::Result<String>;
}
