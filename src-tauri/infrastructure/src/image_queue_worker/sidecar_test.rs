use std::fs;
use std::path::Path;

use super::sidecar::{ExtractIconRunner, ExtractIconRunnerImpl};

#[test]
fn sidecar_存在しないとfalseを返す() {
    let runner = ExtractIconRunnerImpl::new_with_sidecar_path("C:/not-exists/extract-icon.exe");
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("out.png");
    let res = runner
        .extract_icon(48, "C:/not-exists/app.exe", &dst.to_string_lossy())
        .unwrap();
    assert!(!res);
    assert!(!Path::new(&dst).exists());
}

#[test]
fn sidecar_成功でpngを書き出す_クリーンアップ() {
    let sidecar = crate::test_support::ensure_extract_icon_sidecar();
    let runner = ExtractIconRunnerImpl::new_with_sidecar_path(&sidecar);
    let exe = crate::test_support::icon_source_exe();
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("out.png");
    let ok = runner
        .extract_icon(48, &exe.to_string_lossy(), &dst.to_string_lossy())
        .unwrap();
    assert!(ok);
    assert!(Path::new(&dst).exists());
    let _ = fs::remove_file(&dst);
}
