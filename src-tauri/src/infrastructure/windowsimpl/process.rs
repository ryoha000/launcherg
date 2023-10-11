use async_trait::async_trait;

use super::windows::WindowsImpl;
use crate::domain::{process::Process, windows::process::ProcessWindows};

#[async_trait]
impl ProcessWindows for WindowsImpl<Process> {
    fn save_screenshot(&self, process_id: u32, filepath: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
