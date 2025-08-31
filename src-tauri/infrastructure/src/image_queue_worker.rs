use std::path::Path;

use domain::{save_image_queue::{ImageSrcType, ImagePreprocess}};
use crate::icon::process_square_icon;
use crate::thumbnail as thumb_infra;
use domain::repository::RepositoriesExt;
use domain::repository::manager::RepositoryManager;
use domain::native_host_log::{HostLogLevel, HostLogType};
use domain::repository::native_host_log::NativeHostLogRepository;
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::service::save_path_resolver::SavePathResolver;
use std::marker::PhantomData;

pub struct ImageQueueWorker<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: std::sync::Arc<M>,
    resolver: std::sync::Arc<dyn SavePathResolver>,
    _marker: PhantomData<R>,
}

impl<M, R> ImageQueueWorker<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub fn new(manager: std::sync::Arc<M>, resolver: std::sync::Arc<dyn SavePathResolver>) -> Self { Self { manager, resolver, _marker: PhantomData } }

    fn ensure_tmp_file(&self, queue_id: i32, src_url: &str) -> anyhow::Result<String> {
        Ok(self.resolver.tmp_download_path_for_queue(queue_id, src_url))
    }

    pub async fn drain_until_empty(&self) -> anyhow::Result<()> {
        {
            let _ = self.manager.run(|repos| {
                Box::pin(async move {
                    let mut log = repos.host_log();
                    let _ = log.insert_log(HostLogLevel::Debug, HostLogType::ImageQueueWorkerStarted, "image_queue_worker started").await;
                    Ok::<(), anyhow::Error>(())
                })
            }).await;
        }

        loop {
            let items = self.manager.run(|repos| {
                Box::pin(async move {
                    let mut iq = repos.image_queue();
                    iq.list_unfinished_oldest(50).await
                })
            }).await?;
            if items.is_empty() { break; }
            for item in items {
                {
                    let msg = format!("start id={} dst={} src={}", item.id.value, item.dst_path, item.src);
                    let _ = self.manager.run(|repos| {
                        let msg = msg.clone();
                        Box::pin(async move {
                            let mut log = repos.host_log();
                            let _ = log.insert_log(HostLogLevel::Debug, HostLogType::ImageQueueItemStarted, &msg).await;
                            Ok::<(), anyhow::Error>(())
                        })
                    }).await;
                }
                let result: anyhow::Result<()> = async {
                    if Path::new(&item.dst_path).exists() { return Ok(()); }

                    let local_src_path: String = match item.src_type {
                        ImageSrcType::Url => {
                            let tmp = self.ensure_tmp_file(item.id.value, &item.src)?;
                            thumb_infra::download_to_file(&item.src, &tmp).await?;
                            tmp
                        }
                        ImageSrcType::Path => item.src.clone(),
                    };

                    match item.preprocess {
                        ImagePreprocess::ResizeAndCropSquare256 => {
                            process_square_icon(&local_src_path, &item.dst_path, 256)?;
                        }
                        ImagePreprocess::ResizeForWidth400 => {
                            thumb_infra::resize_image(&local_src_path, &item.dst_path, 400)?;
                        }
                        ImagePreprocess::None => {
                            std::fs::copy(Path::new(&local_src_path), Path::new(&item.dst_path)).map_err(|e| anyhow::anyhow!(e))?;
                        }
                    }

                    if matches!(item.src_type, ImageSrcType::Url) {
                        let _ = std::fs::remove_file(Path::new(&local_src_path));
                    }
                    Ok(())
                }.await;

                match result {
                    Ok(_) => {
                        let finished_id = item.id.clone();
                        let finished_id_value = finished_id.value;
                        let _ = self.manager.run(|repos| {
                            Box::pin(async move {
                                let mut iq = repos.image_queue();
                                let _ = iq.mark_finished(finished_id).await;
                                let mut log = repos.host_log();
                                let _ = log.insert_log(HostLogLevel::Debug, HostLogType::ImageQueueItemSucceeded, &format!("done id={}", finished_id_value)).await;
                                Ok::<(), anyhow::Error>(())
                            })
                        }).await;
                    }
                    Err(e) => {
                        let failed_id = item.id.clone();
                        let failed_id_value = failed_id.value;
                        let msg = format!("failed id={} err={}", failed_id_value, e);
                        let _ = self.manager.run(|repos| {
                            let msg = msg.clone();
                            Box::pin(async move {
                                let mut iq = repos.image_queue();
                                let _ = iq.mark_failed(failed_id, &msg).await;
                                let mut log = repos.host_log();
                                let _ = log.insert_log(HostLogLevel::Error, HostLogType::ImageQueueItemFailed, &msg).await;
                                Ok::<(), anyhow::Error>(())
                            })
                        }).await;
                    }
                }
            }
        }

        {
            let _ = self.manager.run(|repos| {
                Box::pin(async move {
                    let mut log = repos.host_log();
                    let _ = log.insert_log(HostLogLevel::Debug, HostLogType::ImageQueueWorkerFinished, "image_queue_worker finished").await;
                    Ok::<(), anyhow::Error>(())
                })
            }).await;
        }
        Ok(())
    }
}
