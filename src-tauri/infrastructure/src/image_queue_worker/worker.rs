use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use domain::repository::RepositoriesExt;
use domain::repository::manager::RepositoryManager;
use domain::save_image_queue::{ImagePreprocess};
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::WindowsExt;

use crate::icon::IconServiceImpl;

use super::preprocess::run_preprocess;
use super::resolver::resolve_source;
use super::types::SourceDecision;

pub struct ImageQueueWorker<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    resolver: Arc<dyn SavePathResolver>,
    windows: Arc<W>,
    _marker: PhantomData<R>,
}

impl<M, R, W> ImageQueueWorker<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>, resolver: Arc<dyn SavePathResolver>, windows: Arc<W>) -> Self { Self { manager, resolver, windows, _marker: PhantomData } }

    pub async fn drain_until_empty(&self) -> anyhow::Result<()> {
        loop {
            let items = self.manager.run(|repos| {
                Box::pin(async move {
                    let mut iq = repos.image_queue();
                    iq.list_unfinished_oldest(50).await
                })
            }).await?;
            if items.is_empty() { break; }
            for item in items {
                let result: anyhow::Result<()> = async {
                    if Path::new(&item.dst_path).exists() { return Ok(()); }

                    // 決定
                    let decision = resolve_source(&*self.windows, &*self.resolver, &item.src, item.src_type).await?;

                    match decision {
                        SourceDecision::FallbackDefaultAndSkip => {
                            IconServiceImpl::write_default_icon(&item.dst_path)?;
                            return Ok(());
                        }
                        SourceDecision::Use(local) => {
                            // 前処理
                            match item.preprocess {
                                ImagePreprocess::ResizeAndCropSquare256 => {
                                    run_preprocess(local.path(), &item.dst_path, ImagePreprocess::ResizeAndCropSquare256)?;
                                }
                                ImagePreprocess::ResizeForWidth400 => {
                                    run_preprocess(local.path(), &item.dst_path, ImagePreprocess::ResizeForWidth400)?;
                                }
                                ImagePreprocess::None => {
                                    run_preprocess(local.path(), &item.dst_path, ImagePreprocess::None)?;
                                }
                            }
                        }
                    }

                    Ok(())
                }.await;

                match result {
                    Ok(_) => {
                        let finished_id = item.id.clone();
                        let _ = self.manager.run(|repos| {
                            Box::pin(async move {
                                let mut iq = repos.image_queue();
                                let _ = iq.mark_finished(finished_id).await;
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
                                Ok::<(), anyhow::Error>(())
                            })
                        }).await;
                    }
                }
            }
        }
        Ok(())
    }
}


