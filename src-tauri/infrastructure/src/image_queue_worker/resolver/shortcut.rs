use std::collections::HashMap;
use std::path::Path;

use domain::file::save_ico_to_png_sync;
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::shell_link::ShellLink as _;
use domain::windows::WindowsExt;

use super::exe;
use crate::image_queue_worker::types::{Cleanup, LocalSource, SourceDecision};

/// ショートカット（.lnk / .url など）由来の画像ソースを解決する。
pub fn resolve<W: WindowsExt>(
    windows: &W,
    resolver: &dyn SavePathResolver,
    src: &str,
) -> anyhow::Result<SourceDecision> {
    let metas = windows
        .shell_link()
        .get_lnk_metadatas(vec![src.to_string()])?;
    decision_from_metadata(resolver, src, &metas)
}

/// 事前に取得済みのメタデータ群から決定を行う
pub fn decision_from_metadata(
    resolver: &dyn SavePathResolver,
    src: &str,
    metas: &HashMap<String, domain::file::LnkMetadata>,
) -> anyhow::Result<SourceDecision> {
    // 対象 src に一致するメタデータがある場合のみ、以降の分岐でアイコンの決定を行う
    if let Some(meta) = metas.get(src) {
        // まず icon を評価（非空なら path は見ない）
        if !meta.icon.trim().is_empty() {
            if let Some(local) = try_resolve_field(resolver, &meta.icon)? {
                return Ok(SourceDecision::Use(local));
            }
            return Ok(SourceDecision::FallbackDefaultAndSkip {
                reason: format!(
                    "shortcut metadata icon is non-empty but cannot resolve: {}",
                    meta.icon
                ),
            });
        }
        // icon が空の場合のみ path を評価
        if !meta.path.trim().is_empty() {
            if let Some(local) = try_resolve_field(resolver, &meta.path)? {
                return Ok(SourceDecision::Use(local));
            }
            return Ok(SourceDecision::FallbackDefaultAndSkip {
                reason: format!(
                    "shortcut metadata path is non-empty but cannot resolve: {}",
                    meta.path
                ),
            });
        }

        // 上記いずれにも該当しない: 利用できるアイコンが無いのでフォールバック（呼び出し側で既定アイコン出力）
        return Ok(SourceDecision::FallbackDefaultAndSkip {
            reason: format!(
                "shortcut metadata has no usable icon (icon empty and path not ico). meta={:?}",
                meta
            ),
        });
    }
    // メタデータ自体が見つからない: フォールバック（呼び出し側で既定アイコン出力）
    Ok(SourceDecision::FallbackDefaultAndSkip {
        reason: "shortcut metadata not found for key".to_string(),
    })
}

/// icon/path の値を統一的に評価し、結果を返す。
/// 空文字の場合は None（未指定）を返し、
/// 非空の場合は Some(Ok/Err) を返す。
fn try_resolve_field(
    resolver: &dyn SavePathResolver,
    value: &str,
) -> anyhow::Result<Option<LocalSource>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let ext = Path::new(trimmed)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());

    let result = match ext.as_deref() {
        Some("ico") => {
            let tmp_png = resolver.tmp_unique_path_with_ext("png");
            save_ico_to_png_sync(trimmed, &tmp_png)?;
            Some(LocalSource::new(tmp_png, Cleanup::None))
        }
        Some("exe") => match exe::resolve(resolver, trimmed)? {
            SourceDecision::Use(local) => Some(local),
            _ => None,
        },
        _ => None,
    };
    Ok(result)
}
