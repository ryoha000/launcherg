use async_trait::async_trait;

#[async_trait]
pub trait ProcessWindows {
    fn save_screenshot_by_process_id(&self, process_id: u32, filepath: &str) -> anyhow::Result<()>;
}
