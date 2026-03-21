use std::sync::Arc;
use std::convert::TryFrom;

use anyhow::Result;
use derive_new::new;
use futures::FutureExt;

use domain::{
    native_host_log::{HostLogLevel, HostLogType},
    pubsub::{
        ImageQueueItemErrorPayload, ImageQueueItemPayload, ImageQueueWorkerStatusPayload,
        PubSubEvent, PubSubService,
    },
    repository::{manager::RepositoryManager, RepositoriesExt},
    save_image_queue::ImageSaveQueueRow,
    service::image_queue_event::ImageQueueWorkerEventHandler,
};

#[derive(Clone)]
pub struct ImageQueueHostLogHandler<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: std::marker::PhantomData<R>,
}

impl<M, R> ImageQueueHostLogHandler<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>) -> Self {
        Self {
            manager,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<M, R> ImageQueueWorkerEventHandler for ImageQueueHostLogHandler<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    fn on_worker_started(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        async move {
            let _ = manager
                .run(|repos| {
                    Box::pin(async move {
                        use domain::repository::native_host_log::NativeHostLogRepository as _;
                        repos
                            .host_log()
                            .insert_log(
                                HostLogLevel::Info,
                                HostLogType::ImageQueueWorkerStarted,
                                "",
                            )
                            .await?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
            Ok(())
        }
        .boxed()
    }
    fn on_worker_finished(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        async move {
            let _ = manager
                .run(|repos| {
                    Box::pin(async move {
                        use domain::repository::native_host_log::NativeHostLogRepository as _;
                        repos
                            .host_log()
                            .insert_log(
                                HostLogLevel::Info,
                                HostLogType::ImageQueueWorkerFinished,
                                "",
                            )
                            .await?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
            Ok(())
        }
        .boxed()
    }
    fn on_item_started(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        let item = item.clone();
        async move {
            let msg = format!(
                "id={} src_type={:?} src=\"{}\" dst=\"{}\"",
                item.id.value, item.src_type, item.src, item.dst_path
            );
            let _ = manager
                .run(|repos| {
                    let msg = msg.clone();
                    Box::pin(async move {
                        use domain::repository::native_host_log::NativeHostLogRepository as _;
                        repos
                            .host_log()
                            .insert_log(
                                HostLogLevel::Info,
                                HostLogType::ImageQueueItemStarted,
                                msg.as_str(),
                            )
                            .await?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
            Ok(())
        }
        .boxed()
    }
    fn on_item_succeeded(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        let item = item.clone();
        async move {
            let msg = format!(
                "id={} src_type={:?} src=\"{}\" dst=\"{}\"",
                item.id.value, item.src_type, item.src, item.dst_path
            );
            let _ = manager
                .run(|repos| {
                    let msg = msg.clone();
                    Box::pin(async move {
                        use domain::repository::native_host_log::NativeHostLogRepository as _;
                        repos
                            .host_log()
                            .insert_log(
                                HostLogLevel::Info,
                                HostLogType::ImageQueueItemSucceeded,
                                msg.as_str(),
                            )
                            .await?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
            Ok(())
        }
        .boxed()
    }
    fn on_item_failed(
        &self,
        item: &ImageSaveQueueRow,
        error_message: &str,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        let item = item.clone();
        let error_message = error_message.to_string();
        async move {
            let msg = format!(
                "id={} src_type={:?} src=\"{}\" dst=\"{}\" err=\"{}\"",
                item.id.value, item.src_type, item.src, item.dst_path, error_message
            );
            let _ = manager
                .run(|repos| {
                    let msg = msg.clone();
                    Box::pin(async move {
                        use domain::repository::native_host_log::NativeHostLogRepository as _;
                        repos
                            .host_log()
                            .insert_log(
                                HostLogLevel::Error,
                                HostLogType::ImageQueueItemFailed,
                                msg.as_str(),
                            )
                            .await?;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
            Ok(())
        }
        .boxed()
    }
}

#[derive(Clone)]
pub struct ImageQueuePubSubHandler<M, R, P>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService + Clone + Send + Sync + 'static,
{
    manager: Arc<M>,
    pubsub: P,
    _marker: std::marker::PhantomData<R>,
}

impl<M, R, P> ImageQueuePubSubHandler<M, R, P>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService + Clone + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>, pubsub: P) -> Self {
        Self {
            manager,
            pubsub,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<M, R, P> ImageQueueWorkerEventHandler for ImageQueuePubSubHandler<M, R, P>
where
    M: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService + Clone + Send + Sync + 'static,
{
    fn on_worker_started(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let manager = Arc::clone(&self.manager);
        let pubsub = self.pubsub.clone();
        async move {
            let total = manager
                .run(|repos| {
                    Box::pin(async move {
                        use domain::repository::save_image_queue::ImageSaveQueueRepository as _;
                        repos.image_queue().count(true).await
                    })
                })
                .await
                .ok()
                .and_then(|count| i32::try_from(count).ok());
            let _ = pubsub.notify(PubSubEvent::ImageQueueWorkerStarted(
                ImageQueueWorkerStatusPayload::new("started".into(), total),
            ));
            Ok(())
        }
        .boxed()
    }
    fn on_worker_finished(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let pubsub = self.pubsub.clone();
        async move {
            let _ = pubsub.notify(PubSubEvent::ImageQueueWorkerFinished(
                ImageQueueWorkerStatusPayload::new("finished".into(), None),
            ));
            Ok(())
        }
        .boxed()
    }
    fn on_item_started(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let pubsub = self.pubsub.clone();
        let item = item.clone();
        async move {
            let payload = ImageQueueItemPayload::new(
                item.id.value.to_string(),
                item.src.clone(),
                item.src_type as i32,
                item.dst_path.clone(),
            );
            let _ = pubsub.notify(PubSubEvent::ImageQueueItemStarted(payload));
            Ok(())
        }
        .boxed()
    }
    fn on_item_succeeded(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let pubsub = self.pubsub.clone();
        let item = item.clone();
        async move {
            let payload = ImageQueueItemPayload::new(
                item.id.value.to_string(),
                item.src.clone(),
                item.src_type as i32,
                item.dst_path.clone(),
            );
            let _ = pubsub.notify(PubSubEvent::ImageQueueItemSucceeded(payload));
            Ok(())
        }
        .boxed()
    }
    fn on_item_failed(
        &self,
        item: &ImageSaveQueueRow,
        error_message: &str,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let pubsub = self.pubsub.clone();
        let item = item.clone();
        let error_message = error_message.to_string();
        async move {
            let payload = ImageQueueItemErrorPayload::new(item.id.value.to_string(), error_message);
            let _ = pubsub.notify(PubSubEvent::ImageQueueItemFailed(payload));
            Ok(())
        }
        .boxed()
    }
}

#[derive(new, Clone)]
pub struct ImageQueueCompositeHandler {
    handlers: Vec<Arc<dyn ImageQueueWorkerEventHandler + Send + Sync>>,
}

impl ImageQueueWorkerEventHandler for ImageQueueCompositeHandler {
    fn on_worker_started(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let handlers = self.handlers.clone();
        async move {
            for h in &handlers {
                let _ = h.on_worker_started().await;
            }
            Ok(())
        }
        .boxed()
    }
    fn on_worker_finished(&self) -> futures::future::BoxFuture<'static, Result<()>> {
        let handlers = self.handlers.clone();
        async move {
            for h in &handlers {
                let _ = h.on_worker_finished().await;
            }
            Ok(())
        }
        .boxed()
    }
    fn on_item_started(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let handlers = self.handlers.clone();
        let item = item.clone();
        async move {
            for h in &handlers {
                let _ = h.on_item_started(&item).await;
            }
            Ok(())
        }
        .boxed()
    }
    fn on_item_succeeded(
        &self,
        item: &ImageSaveQueueRow,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let handlers = self.handlers.clone();
        let item = item.clone();
        async move {
            for h in &handlers {
                let _ = h.on_item_succeeded(&item).await;
            }
            Ok(())
        }
        .boxed()
    }
    fn on_item_failed(
        &self,
        item: &ImageSaveQueueRow,
        error_message: &str,
    ) -> futures::future::BoxFuture<'static, Result<()>> {
        let handlers = self.handlers.clone();
        let item = item.clone();
        let error_message = error_message.to_string();
        async move {
            for h in &handlers {
                let _ = h.on_item_failed(&item, &error_message).await;
            }
            Ok(())
        }
        .boxed()
    }
}
