use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet};

use domain::repository::manager::RepositoryManager;
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::repository::RepositoriesExt;
use domain::save_image_queue::ImageSrcType;
use domain::service::image_queue_event::ImageQueueWorkerEventHandler;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::shell_link::ShellLink as _;
use domain::windows::WindowsExt;

use crate::icon::IconServiceImpl;

use super::preprocess::run_preprocess;
use super::resolver::resolve_source_with_shortcut_metas;
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
    event_handler: Option<Arc<dyn ImageQueueWorkerEventHandler + Send + Sync>>,
    _marker: PhantomData<R>,
}

impl<M, R, W> ImageQueueWorker<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    pub fn new(manager: Arc<M>, resolver: Arc<dyn SavePathResolver>, windows: Arc<W>) -> Self {
        Self {
            manager,
            resolver,
            windows,
            event_handler: None,
            _marker: PhantomData,
        }
    }

    pub fn new_with_event_handler(
        manager: Arc<M>,
        resolver: Arc<dyn SavePathResolver>,
        windows: Arc<W>,
        event_handler: Arc<dyn ImageQueueWorkerEventHandler + Send + Sync>,
    ) -> Self {
        Self {
            manager,
            resolver,
            windows,
            event_handler: Some(event_handler),
            _marker: PhantomData,
        }
    }

    pub async fn drain_until_empty(&self) -> anyhow::Result<()> {
        if let Some(handler) = &self.event_handler {
            let h = Arc::clone(handler);
            let _ = h.on_worker_started().await;
        }
        loop {
            let items = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut iq = repos.image_queue();
                        iq.list(true, 50).await
                    })
                })
                .await?;
            if items.is_empty() {
                break;
            }
            // 1) バッチ内ショートカットのパスをユニーク収集
            let shortcut_srcs: Vec<String> = {
                let set: HashSet<String> = items
                    .iter()
                    .filter(|it| {
                        matches!(it.src_type, ImageSrcType::Shortcut)
                            && !Path::new(&it.dst_path).exists()
                    })
                    .map(|it| it.src.clone())
                    .collect();
                set.into_iter().collect()
            };

            // 2) get_lnk_metadatas を OS スレッドで一括実行（必要時のみ）
            let shortcut_metas: HashMap<String, domain::file::LnkMetadata> =
                if shortcut_srcs.is_empty() {
                    HashMap::new()
                } else {
                    let windows = Arc::clone(&self.windows);
                    tokio::task::spawn_blocking(move || {
                        windows.shell_link().get_lnk_metadatas(shortcut_srcs)
                    })
                    .await??
                };
            let shortcut_metas = Arc::new(shortcut_metas);

            // 3) 並列処理（前処理/デフォルトアイコンは spawn_blocking）
            let concurrency: usize = 8;
            stream::iter(items)
                .for_each_concurrent(concurrency, |item| {
                    let manager = Arc::clone(&self.manager);
                    let resolver = Arc::clone(&self.resolver);
                    let windows = Arc::clone(&self.windows);
                    let shortcut_metas = Arc::clone(&shortcut_metas);
                    let event_handler = self.event_handler.as_ref().map(Arc::clone);

                    async move {
                        let result: anyhow::Result<()> = async {
                            if let Some(h) = &event_handler {
                                let _ = h.on_item_started(&item).await;
                            }
                            if Path::new(&item.dst_path).exists() {
                                return Ok(());
                            }

                            // 決定（ショートカットは事前メタを使用して追加の COM 呼び出しを避ける）
                            let decision = resolve_source_with_shortcut_metas(
                                &*windows,
                                &*resolver,
                                &item.src,
                                item.src_type,
                                Some(&*shortcut_metas),
                            )
                            .await?;

                            match decision {
                                SourceDecision::FallbackDefaultAndSkip { reason } => {
                                    let dst = item.dst_path.clone();
                                    tokio::task::spawn_blocking(move || {
                                        IconServiceImpl::write_default_icon(&dst)
                                    })
                                    .await??;
                                    // フォールバックは成功扱いではなく、エラーとして記録する
                                    let msg = format!(
                                        "fallback: {}; wrote default icon. src={} src_type={:?}",
                                        reason, item.src, item.src_type
                                    );
                                    return Err(anyhow::anyhow!(msg));
                                }
                                SourceDecision::Use(local) => {
                                    let src_path = local.path().to_string();
                                    let dst_path = item.dst_path.clone();
                                    let preprocess = item.preprocess;
                                    tokio::task::spawn_blocking(move || {
                                        run_preprocess(&src_path, &dst_path, preprocess)
                                    })
                                    .await??;
                                }
                            }

                            Ok(())
                        }
                        .await;

                        match result {
                            Ok(_) => {
                                let finished_id = item.id.clone();
                                let _ = manager
                                    .run(|repos| {
                                        Box::pin(async move {
                                            let mut iq = repos.image_queue();
                                            let _ = iq.mark_finished(finished_id).await;
                                            Ok::<(), anyhow::Error>(())
                                        })
                                    })
                                    .await;
                                if let Some(h) = &event_handler {
                                    let _ = h.on_item_succeeded(&item).await;
                                }
                            }
                            Err(e) => {
                                let failed_id = item.id.clone();
                                let failed_id_value = failed_id.value;
                                let msg = format!("failed id={} err={:#}", failed_id_value, e);
                                let _ = manager
                                    .run(|repos| {
                                        let msg = msg.clone();
                                        Box::pin(async move {
                                            let mut iq = repos.image_queue();
                                            let _ = iq.mark_failed(failed_id, &msg).await;
                                            Ok::<(), anyhow::Error>(())
                                        })
                                    })
                                    .await;
                                if let Some(h) = &event_handler {
                                    let _ = h.on_item_failed(&item, &msg).await;
                                }
                            }
                        }
                    }
                })
                .await;
        }
        if let Some(handler) = &self.event_handler {
            let h = Arc::clone(handler);
            let _ = h.on_worker_finished().await;
        }
        Ok(())
    }
}
