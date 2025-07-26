use std::sync::Arc;

use derive_new::new;

use crate::{
    domain::windows::process::ProcessWindows, 
    domain::windows::proctail::{ProcTail, ProcTailEvent, ServiceStatus, WatchTarget, HealthCheckResult},
    infrastructure::windowsimpl::windows::WindowsExt,
    infrastructure::windowsimpl::proctail_manager::{ProcTailManagerStatus, ProcTailVersion},
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

    // ProcTail Manager methods
    pub async fn proctail_manager_get_status(&self) -> anyhow::Result<ProcTailManagerStatus> {
        Ok(self.windows
            .proctail_manager()
            .get_status()
            .await?)
    }

    pub async fn proctail_manager_get_latest_version(&self) -> anyhow::Result<ProcTailVersion> {
        Ok(self.windows
            .proctail_manager()
            .get_latest_version()
            .await?)
    }

    pub async fn proctail_manager_is_update_available(&self) -> anyhow::Result<bool> {
        Ok(self.windows
            .proctail_manager()
            .is_update_available()
            .await?)
    }

    pub async fn proctail_manager_download_and_install(&self) -> anyhow::Result<()> {
        let latest_version = self.windows
            .proctail_manager()
            .get_latest_version()
            .await?;
        
        self.windows
            .proctail_manager()
            .download_and_install(&latest_version)
            .await?;
        
        Ok(())
    }

    pub async fn proctail_manager_start(&self) -> anyhow::Result<()> {
        Ok(self.windows
            .proctail_manager()
            .start_proctail()
            .await?)
    }

    pub async fn proctail_manager_stop(&self) -> anyhow::Result<()> {
        Ok(self.windows
            .proctail_manager()
            .stop_proctail()
            .await?)
    }

    pub async fn proctail_manager_is_running(&self) -> anyhow::Result<bool> {
        Ok(self.windows
            .proctail_manager()
            .is_running()
            .await)
    }
}
