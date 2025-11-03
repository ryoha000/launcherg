use std::sync::Arc;

use derive_new::new;
use domain::repository::{manager::RepositoryManager, works::WorkRepository as _, RepositoriesExt};
use domain::service::save_path_resolver::SavePathResolver;
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkThumbnailUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    resolver: Arc<dyn SavePathResolver>,
    _marker: PhantomData<R>,
}

impl<M, R> WorkThumbnailUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn backfill_thumbnail_sizes(&self) -> anyhow::Result<usize> {
        let resolver = self.resolver.clone();
        let updated = self
            .manager
            .run(|repos| {
                let resolver = resolver.clone();
                Box::pin(async move {
                    let mut work_repo = repos.work();
                    let ids = work_repo.list_work_ids_missing_thumbnail_size().await?;
                    let mut updated: usize = 0;
                    if !ids.is_empty() {
                        for id in ids.into_iter() {
                            let path = resolver.thumbnail_png_path(&id.value);
                            match image::image_dimensions(&path) {
                                Ok((w, h)) => {
                                    let _ = work_repo
                                        .upsert_work_thumbnail_size(id, w as i32, h as i32)
                                        .await;
                                    updated += 1;
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Ok::<usize, anyhow::Error>(updated)
                })
            })
            .await?;
        Ok(updated)
    }
}
