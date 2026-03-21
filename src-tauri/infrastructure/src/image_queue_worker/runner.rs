use std::sync::Arc;

use domain::repository::{manager::RepositoryManager, RepositoriesExt};
use domain::service::image_queue_drain::ImageQueueDrainService;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::WindowsExt;

use super::ImageQueueWorker;
use domain::service::image_queue_event::ImageQueueWorkerEventHandler;

pub struct ImageQueueRunnerImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    worker: Arc<ImageQueueWorker<M, R, W>>,
    is_running: std::sync::atomic::AtomicBool,
}

impl<M, R, W> ImageQueueRunnerImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>, resolver: Arc<dyn SavePathResolver>, windows: Arc<W>) -> Self {
        let worker = Arc::new(ImageQueueWorker::new(manager, resolver, windows));
        Self {
            worker,
            is_running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn new_with_event_handler(
        manager: Arc<M>,
        resolver: Arc<dyn SavePathResolver>,
        windows: Arc<W>,
        handler: Arc<dyn ImageQueueWorkerEventHandler + Send + Sync>,
    ) -> Self {
        let worker = Arc::new(ImageQueueWorker::new_with_event_handler(
            manager, resolver, windows, handler,
        ));
        Self {
            worker,
            is_running: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl<M, R, W> ImageQueueDrainService for ImageQueueRunnerImpl<M, R, W>
where
    M: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    async fn drain_until_empty(&self) -> anyhow::Result<()> {
        // Non-blocking single-flight guard
        if self
            .is_running
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            return Ok(());
        }
        let result = self.worker.drain_until_empty().await;
        self.is_running
            .store(false, std::sync::atomic::Ordering::Release);
        result
    }
}
