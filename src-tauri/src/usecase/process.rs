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
    pub async fn save_screenshot_by_pid(
        &self,
        process_id: u32,
        filepath: &str,
    ) -> anyhow::Result<()> {
        self.windows
            .process()
            .save_screenshot_by_process_id(process_id, &filepath)
    }
}
