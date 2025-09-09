use std::collections::HashMap;

use domain::file::save_ico_to_png_sync;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::WindowsExt;
use domain::windows::shell_link::ShellLink as _;

use crate::image_queue_worker::types::{LocalSource, SourceDecision, Cleanup};

/// ショートカット（.lnk / .url など）由来の画像ソースを解決する。
pub fn resolve<W: WindowsExt>(
    windows: &W,
    resolver: &dyn SavePathResolver,
    src: &str,
) -> anyhow::Result<SourceDecision> {
    let metas = windows.shell_link().get_lnk_metadatas(vec![src.to_string()])?;
    if let Some(meta) = metas.get(src) {
        let icon_lower = meta.icon.to_lowercase();
        if !icon_lower.is_empty() && icon_lower.ends_with("ico") {
            let tmp_png = resolver.tmp_unique_path_with_ext("png");
            let _ = save_ico_to_png_sync(&meta.icon, &tmp_png);
            return Ok(SourceDecision::Use(LocalSource::new(tmp_png, Cleanup::None)));
        } else if !meta.icon.is_empty() {
            return Ok(SourceDecision::Use(LocalSource::new(meta.icon.clone(), Cleanup::None)));
        } else if meta.path.to_lowercase().ends_with("ico") {
            let tmp_png = resolver.tmp_unique_path_with_ext("png");
            let _ = save_ico_to_png_sync(&meta.path, &tmp_png);
            return Ok(SourceDecision::Use(LocalSource::new(tmp_png, Cleanup::None)));
        } else {
            return Ok(SourceDecision::FallbackDefaultAndSkip);
        }
    }
    Ok(SourceDecision::FallbackDefaultAndSkip)
}

/// 事前に取得済みのメタデータ群から決定を行う
pub fn decision_from_metadata(
    resolver: &dyn SavePathResolver,
    src: &str,
    metas: &HashMap<String, domain::file::LnkMetadata>,
) -> anyhow::Result<SourceDecision> {
    if let Some(meta) = metas.get(src) {
        let icon_lower = meta.icon.to_lowercase();
        if !icon_lower.is_empty() && icon_lower.ends_with("ico") {
            let tmp_png = resolver.tmp_unique_path_with_ext("png");
            let _ = save_ico_to_png_sync(&meta.icon, &tmp_png);
            return Ok(SourceDecision::Use(LocalSource::new(tmp_png, Cleanup::None)));
        } else if !meta.icon.is_empty() {
            return Ok(SourceDecision::Use(LocalSource::new(meta.icon.clone(), Cleanup::None)));
        } else if meta.path.to_lowercase().ends_with("ico") {
            let tmp_png = resolver.tmp_unique_path_with_ext("png");
            let _ = save_ico_to_png_sync(&meta.path, &tmp_png);
            return Ok(SourceDecision::Use(LocalSource::new(tmp_png, Cleanup::None)));
        } else {
            return Ok(SourceDecision::FallbackDefaultAndSkip);
        }
    }
    Ok(SourceDecision::FallbackDefaultAndSkip)
}


