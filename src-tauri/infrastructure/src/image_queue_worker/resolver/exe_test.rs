use std::fs;
use std::path::Path;

use domain::service::save_path_resolver::DirsSavePathResolver;

use super::exe::resolve;
use crate::image_queue_worker::types::SourceDecision;

#[test]
fn exe_存在しない場合は_skip() {
    let resolver = DirsSavePathResolver::default();
    let res = resolve(&resolver, "C:/not-exists/app.exe").unwrap();
    match res { SourceDecision::FallbackDefaultAndSkip => {}, _ => panic!("expected skip") }
}

#[test]
fn exe_sidecar成功時_pngの一時パスが返る_クリーンアップ() {
    // Arrange: 配置先は current_exe と同ディレクトリ
    let sidecar_dst = {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .expect("resolve current exe dir");
        let dst = exe_dir.join("extract-icon.exe");
        const BYTES: &[u8] = include_bytes!("../../../../bin/extract-icon-x86_64-pc-windows-msvc.exe");
        fs::write(&dst, BYTES).expect("write sidecar");
        dst
    };

    // 実行対象のexe（リポジトリのアセット）
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let exe = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("image_queue_worker")
        .join("assets")
        .join("dohnadohna.exe");

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
    let _ = fs::remove_file(&sidecar_dst);
}


