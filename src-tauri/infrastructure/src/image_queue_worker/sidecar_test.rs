use std::fs;
use std::path::Path;

use super::sidecar::{ExtractIconRunner, ExtractIconRunnerImpl};

fn write_sidecar_in_exe_dir_if_needed() -> (std::path::PathBuf, bool) {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .expect("resolve current exe dir");
    let sidecar_path = exe_dir.join("extract-icon.exe");
    if sidecar_path.exists() {
        return (sidecar_path, false);
    }
    const BYTES: &[u8] = include_bytes!("../../../bin/extract-icon-x86_64-pc-windows-msvc.exe");
    fs::write(&sidecar_path, BYTES).expect("write sidecar");
    (sidecar_path, true)
}

fn asset_exe_path() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest_dir)
        .join("src")
        .join("image_queue_worker")
        .join("assets")
        .join("dohnadohna.exe")
}

#[test]
fn sidecar_存在しないとfalseを返す() {
    let runner = ExtractIconRunnerImpl::new_with_sidecar_path("C:/not-exists/extract-icon.exe");
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("out.png");
    let res = runner.extract_icon(48, "C:/not-exists/app.exe", &dst.to_string_lossy()).unwrap();
    assert!(!res);
    assert!(!Path::new(&dst).exists());
}

#[test]
fn sidecar_成功でpngを書き出す_クリーンアップ() {
    let (sidecar, created) = write_sidecar_in_exe_dir_if_needed();
    let runner = ExtractIconRunnerImpl::new_with_sidecar_path(&sidecar);
    let exe = asset_exe_path();
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("out.png");
    let ok = runner.extract_icon(48, &exe.to_string_lossy(), &dst.to_string_lossy()).unwrap();
    assert!(ok);
    assert!(Path::new(&dst).exists());
    let _ = fs::remove_file(&dst);
    if created { let _ = fs::remove_file(&sidecar); }
}


