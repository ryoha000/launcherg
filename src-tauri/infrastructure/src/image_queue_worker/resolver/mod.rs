use domain::save_image_queue::ImageSrcType;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::WindowsExt;

use crate::image_queue_worker::types::{LocalSource, SourceDecision, Cleanup};

pub mod url;
pub mod path;
pub mod shortcut;
pub mod exe;
#[cfg(test)]
mod url_test;
#[cfg(test)]
mod path_test;
#[cfg(test)]
mod shortcut_test;
#[cfg(test)]
mod exe_test;

pub async fn resolve_source<W: WindowsExt>(
    windows: &W,
    resolver: &dyn SavePathResolver,
    src: &str,
    src_type: ImageSrcType,
) -> anyhow::Result<SourceDecision> {
    Ok(match src_type {
        ImageSrcType::Url => {
            let tmp = url::resolve_to_tmp(resolver, src).await?;
            let cleanup_path = tmp.clone();
            SourceDecision::Use(LocalSource::new(tmp, Cleanup::DeleteOnDrop { path: cleanup_path }))
        }
        ImageSrcType::Path => {
            SourceDecision::Use(LocalSource::new(path::resolve(src), Cleanup::None))
        }
        ImageSrcType::Shortcut => shortcut::resolve(windows, resolver, src)?,
        ImageSrcType::Exe => exe::resolve(resolver, src)?,
    })
}


