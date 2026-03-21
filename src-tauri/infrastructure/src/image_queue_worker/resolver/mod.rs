use domain::save_image_queue::ImageSrcType;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::WindowsExt;

use crate::image_queue_worker::types::{Cleanup, LocalSource, SourceDecision};

pub mod exe;
#[cfg(test)]
mod exe_test;
pub mod path;
#[cfg(test)]
mod path_test;
pub mod shortcut;
#[cfg(test)]
mod shortcut_test;
pub mod url;
#[cfg(test)]
mod url_test;

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
            SourceDecision::Use(LocalSource::new(
                tmp,
                Cleanup::DeleteOnDrop { path: cleanup_path },
            ))
        }
        ImageSrcType::Path => {
            SourceDecision::Use(LocalSource::new(path::resolve(src), Cleanup::None))
        }
        ImageSrcType::Shortcut => shortcut::resolve(windows, resolver, src)?,
        ImageSrcType::Exe => exe::resolve(resolver, src)?,
    })
}

/// 事前取得したショートカットメタデータを活用して解決するバリアント
pub async fn resolve_source_with_shortcut_metas<W: WindowsExt>(
    windows: &W,
    resolver: &dyn SavePathResolver,
    src: &str,
    src_type: ImageSrcType,
    shortcut_metas: Option<&std::collections::HashMap<String, domain::file::LnkMetadata>>,
) -> anyhow::Result<SourceDecision> {
    Ok(match src_type {
        ImageSrcType::Url => {
            let tmp = url::resolve_to_tmp(resolver, src).await?;
            let cleanup_path = tmp.clone();
            SourceDecision::Use(LocalSource::new(
                tmp,
                Cleanup::DeleteOnDrop { path: cleanup_path },
            ))
        }
        ImageSrcType::Path => {
            SourceDecision::Use(LocalSource::new(path::resolve(src), Cleanup::None))
        }
        ImageSrcType::Shortcut => {
            if let Some(metas) = shortcut_metas {
                shortcut::decision_from_metadata(resolver, src, metas)?
            } else {
                shortcut::resolve(windows, resolver, src)?
            }
        }
        ImageSrcType::Exe => exe::resolve(resolver, src)?,
    })
}
