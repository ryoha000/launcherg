use std::sync::Arc;

use derive_new::new;

use crate::domain::{collection::CollectionElement, thumbnail::ThumbnailService, Id};
use tauri::AppHandle;
use crate::infrastructure::util::get_save_root_abs_dir_with_ptr_handle;

#[derive(new)]
pub struct ImageUseCase<S: ThumbnailService> {
    thumbnail_service: Arc<S>,
}

impl<S: ThumbnailService> ImageUseCase<S> {
    pub async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
        self.thumbnail_service.save_thumbnail(id, url).await
    }

    pub async fn concurency_save_thumbnails(
        &self,
        args: Vec<(Id<CollectionElement>, String)>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;
        futures::stream::iter(args.into_iter())
            .map(|(id, url)| async move { self.thumbnail_service.save_thumbnail(&id, &url).await })
            .buffered(50)
            .for_each(|res| async move {
                if let Err(e) = res {
                    eprintln!("[concurency_save_thumbnails] {}", e);
                }
            })
            .await;
        Ok(())
    }
}

// App側で画像保存に用いるルートディレクトリを返す
pub fn get_image_root_dir(handle: &AppHandle) -> String {
    get_save_root_abs_dir_with_ptr_handle(handle)
}
