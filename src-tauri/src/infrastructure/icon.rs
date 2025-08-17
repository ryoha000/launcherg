use std::{path::Path, fs};

use async_trait::async_trait;
use tauri::AppHandle;
use std::sync::Arc;

use crate::domain::{collection::CollectionElement, Id, icon::IconService};
use crate::domain::file::{save_icon_to_png as domain_save_icon_to_png, get_icon_path as domain_get_icon_path};

enum Backend {
    Tauri(Arc<AppHandle>),
    Host { root_dir: String },
}

pub struct IconServiceImpl {
    backend: Backend,
}

impl IconServiceImpl {
    pub fn new_from_app_handle(handle: Arc<AppHandle>) -> Self { Self { backend: Backend::Tauri(handle) } }
    pub fn new_from_root_path(root_dir: String) -> Self { Self { backend: Backend::Host { root_dir } } }

    fn build_icon_path_host(root_dir: &str, id: &Id<CollectionElement>) -> anyhow::Result<String> {
        let dir = Path::new(root_dir).join("game-icons");
        fs::create_dir_all(&dir).ok();
        Ok(dir.join(format!("{}.png", id.value)).to_string_lossy().to_string())
    }

    fn write_default_icon(save_path: &str) -> anyhow::Result<()> {
        let bytes = include_bytes!("..\\..\\icons\\notfound.png");
        let mut file = std::fs::File::create(save_path)?;
        use std::io::Write as _;
        file.write_all(bytes)?;
        Ok(())
    }
}

#[async_trait]
impl IconService for IconServiceImpl {
    async fn save_icon_from_path(&self, id: &Id<CollectionElement>, source_path: &str) -> anyhow::Result<()> {
        match &self.backend {
            Backend::Tauri(handle) => {
                let _ = domain_save_icon_to_png(handle, source_path, id)?.await??;
                Ok(())
            }
            Backend::Host { root_dir } => {
                let save_path = Self::build_icon_path_host(root_dir, id)?;
                // Host では PNG のみ受け付け、それ以外はフォールバック
                if source_path.to_lowercase().ends_with("png") {
                    match std::fs::copy(source_path, &save_path) {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            log::warn!("copy png failed: {}", e);
                            Self::write_default_icon(&save_path)
                        }
                    }
                } else {
                    // fallback: デフォルト
                    Self::write_default_icon(&save_path)
                }
            }
        }
    }

    async fn save_default_icon(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        match &self.backend {
            Backend::Tauri(handle) => {
                let save_path = domain_get_icon_path(handle, id);
                Self::write_default_icon(&save_path)
            }
            Backend::Host { root_dir } => {
                let save_path = Self::build_icon_path_host(root_dir, id)?;
                Self::write_default_icon(&save_path)
            }
        }
    }
}


