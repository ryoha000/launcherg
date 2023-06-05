use async_trait::async_trait;

#[async_trait]
pub trait FileExplorer {
    fn save_base64_image(&self, path: &str, data: String) -> anyhow::Result<()>;
    fn get_save_image_path(&self, id: i32) -> anyhow::Result<String>;
    fn get_md_path(&self, id: i32) -> anyhow::Result<String>;
}
