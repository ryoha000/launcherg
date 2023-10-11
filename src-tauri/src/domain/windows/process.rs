use async_trait::async_trait;

#[async_trait]
pub trait ProcessWindows {
    fn save_screenshot(&self, process_id: u32, filepath: &str) -> anyhow::Result<()>;
}
