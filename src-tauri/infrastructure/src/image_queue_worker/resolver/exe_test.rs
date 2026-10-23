use std::fs;
use std::path::Path;

use domain::service::save_path_resolver::DirsSavePathResolver;

use super::exe::resolve;
use crate::image_queue_worker::types::SourceDecision;

#[test]
fn exe_存在しない場合は_skip() {
    let resolver = DirsSavePathResolver::default();
    let res = resolve(&resolver, "C:/not-exists/app.exe").unwrap();
    match res {
        SourceDecision::FallbackDefaultAndSkip { .. } => {}
        _ => panic!("expected skip"),
    }
}

#[test]
fn exe_sidecar成功時_pngの一時パスが返る_クリーンアップ() {
    crate::test_support::ensure_extract_icon_sidecar();
    let exe = crate::test_support::icon_source_exe();

    let resolver = DirsSavePathResolver::default();

    // Act
    let res = resolve(&resolver, &exe.to_string_lossy()).unwrap();

    // Assert & cleanup
    match res {
        SourceDecision::Use(local) => {
            assert!(Path::new(local.path()).exists());
            let _ = fs::remove_file(local.path());
        }
        _ => panic!("expected Use(local)"),
    }
}
