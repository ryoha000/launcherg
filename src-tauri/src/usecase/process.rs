use std::sync::Arc;

use derive_new::new;

use crate::{
    domain::windows::process::ProcessWindows, 
    domain::windows::proctail::{ProcTail, ProcTailEvent, ServiceStatus, WatchTarget, HealthCheckResult},
    infrastructure::windowsimpl::windows::WindowsExt,
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

    // ProcTail methods
    pub async fn proctail_add_watch_target(
        &self,
        process_id: u32,
        tag: &str,
    ) -> anyhow::Result<WatchTarget> {
        Ok(self.windows
            .proctail()
            .add_watch_target(process_id, tag)
            .await?)
    }

    pub async fn proctail_remove_watch_target(
        &self,
        tag: &str,
    ) -> anyhow::Result<u32> {
        Ok(self.windows
            .proctail()
            .remove_watch_target(tag)
            .await?)
    }

    pub async fn proctail_get_watch_targets(&self) -> anyhow::Result<Vec<WatchTarget>> {
        Ok(self.windows
            .proctail()
            .get_watch_targets()
            .await?)
    }

    pub async fn proctail_get_recorded_events(
        &self,
        tag: &str,
        count: Option<u32>,
        event_type: Option<&str>,
    ) -> anyhow::Result<Vec<ProcTailEvent>> {
        Ok(self.windows
            .proctail()
            .get_recorded_events(tag, count, event_type)
            .await?)
    }

    pub async fn proctail_clear_events(&self, tag: &str) -> anyhow::Result<u32> {
        Ok(self.windows
            .proctail()
            .clear_events(tag)
            .await?)
    }

    pub async fn proctail_get_status(&self) -> anyhow::Result<ServiceStatus> {
        Ok(self.windows
            .proctail()
            .get_status()
            .await?)
    }

    pub async fn proctail_health_check(&self) -> anyhow::Result<HealthCheckResult> {
        Ok(self.windows
            .proctail()
            .health_check()
            .await?)
    }

    pub async fn proctail_is_service_available(&self) -> anyhow::Result<bool> {
        Ok(self.windows
            .proctail()
            .is_service_available()
            .await)
    }
}
