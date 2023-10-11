use std::sync::Arc;

use derive_new::new;

use crate::{
    domain::windows::process::ProcessWindows, infrastructure::windowsimpl::windows::WindowsExt,
};

#[derive(new)]
pub struct ProcessUseCase<R: WindowsExt> {
    windows: Arc<R>,
}

impl<R: WindowsExt> ProcessUseCase<R> {
    pub async fn save_screenshot_by_pid(&self, process_id: u32) -> anyhow::Result<String> {
        let filepath = format!("{}.png", process_id);
        self.windows
            .process()
            .save_screenshot(process_id, &filepath)?;
        Ok(filepath)
    }
}
