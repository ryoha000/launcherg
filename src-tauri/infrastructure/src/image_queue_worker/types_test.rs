use std::path::Path;

use super::types::{Cleanup, LocalSource};

fn write_dummy(path: &str) {
    let p = std::path::Path::new(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, b"x").unwrap();
}

#[test]
fn delete_on_drop_スコープ終了で削除される() {
    // Arrange
    let tmp_dir = tempfile::tempdir().unwrap();
    let p = tmp_dir.path().join("will-delete.bin");
    write_dummy(&p.to_string_lossy());
    assert!(Path::new(&p).exists());

    // Act
    {
        let _local = LocalSource::new(p.to_string_lossy(), Cleanup::DeleteOnDrop { path: p.to_string_lossy().to_string() });
        // drop at scope end
    }

    // Assert
    assert!(!Path::new(&p).exists());
}

#[test]
fn local_source_path_が取得できる() {
    let p = "/tmp/some-file.bin";
    let local = LocalSource::new(p, Cleanup::None);
    assert_eq!(local.path(), p);
}


