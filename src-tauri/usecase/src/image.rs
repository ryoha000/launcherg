use std::sync::Arc;

use derive_new::new;

use domain::{collection::CollectionElement, thumbnail::ThumbnailService, icon::IconService, Id};
use tauri::AppHandle;
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

#[derive(new)]
pub struct ImageUseCase<TS: ThumbnailService, IS: IconService> {
    thumbnail_service: Arc<TS>,
    icon_service: Arc<IS>,
    resolver: Arc<dyn SavePathResolver>,
}

impl<TS: ThumbnailService, IS: IconService> ImageUseCase<TS, IS> {
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

    // icon
    async fn save_icon_from_path(&self, id: &Id<CollectionElement>, source_path: &str) -> anyhow::Result<()> {
        self.icon_service.save_icon_from_path(id, source_path).await
    }

    async fn save_default_icon(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        self.icon_service.save_default_icon(id).await
    }

    // exe/lnk の情報からアイコン保存元を決定し保存する
    pub async fn save_icon_by_paths(
        &self,
        id: &Id<CollectionElement>,
        exe_path: &Option<String>,
        lnk_path: &Option<String>,
    ) -> anyhow::Result<()> {
        let mut icon_source: Option<String> = None;
        if let Some(path) = exe_path.as_ref() {
            icon_source = Some(path.clone());
        } else if let Some(lnk) = lnk_path.as_ref() {
            use domain::file::get_lnk_metadatas;
            let metadatas = get_lnk_metadatas(vec![lnk.as_str()])?;
            if let Some(metadata) = metadatas.get(lnk.as_str()) {
                if metadata.icon.to_lowercase().ends_with("ico") {
                    icon_source = Some(metadata.icon.clone());
                } else {
                    icon_source = Some(metadata.path.clone());
                }
            }
        }
        if let Some(src) = icon_source {
            self.save_icon_from_path(id, &src).await
        } else {
            self.save_default_icon(id).await
        }
    }

    // 既存PNGを上書き（ユーザー指定PNG用）
    pub async fn overwrite_icon_png(&self, id: &Id<CollectionElement>, png_path: &str) -> anyhow::Result<()> {
        let dst = self.resolver.icon_png_path(id.value);
        std::fs::copy(png_path, dst)?;
        Ok(())
    }
}

// App側で画像保存に用いるルートディレクトリを返す
pub fn get_image_root_dir(_handle: &AppHandle) -> String {
    DirsSavePathResolver::default().root_dir()
}
