use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::explored_cache::ExploredCache;
use domain::scan::{CandidateKind, FileSystem};

use super::LocalFileSystem;

fn create_file(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"").unwrap();
}

fn create_dir(path: &Path) { fs::create_dir_all(path).unwrap(); }

fn collect_set(
    fs_impl: &LocalFileSystem,
    roots: &[PathBuf],
    exclude: Option<ExploredCache>,
) -> HashSet<(String, CandidateKind)> {
    let iter = fs_impl
        .walk_dir(roots, exclude.map(Arc::new))
        .expect("walk_dir should succeed");
    iter
        .map(|c| (c.path.to_string_lossy().to_string(), c.kind))
        .collect()
}

#[test]
fn walk_dir_テーブルテスト() {
    // Arrange (各ケースごとのArrangeは build 関数内で行う)
    struct Case {
        name: &'static str,
        build: fn() -> (tempfile::TempDir, Vec<PathBuf>, Option<ExploredCache>, HashSet<(String, CandidateKind)>),
    }

    let cases: Vec<Case> = vec![
        Case { // 基本列挙（exe/lnk/その他）
            name: "基本列挙_拡張子とディレクトリを判定する",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let p_exe = root.join("game.exe");
                let p_lnk = root.join("shortcut.LNK");
                let p_dir = root.join("folder");
                let p_other = root.join("text.txt");
                create_file(&p_exe);
                create_file(&p_lnk);
                create_dir(&p_dir);
                create_file(&p_other);
                let expected: HashSet<(String, CandidateKind)> = vec![
                    (p_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                    (p_lnk.to_string_lossy().to_string(), CandidateKind::Shortcut),
                ].into_iter().collect();
                (tmp, vec![root], None, expected)
            },
        },
        Case {
            name: "再帰的にディレクトリを探索する",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let p_root_exe = root.join("game.exe");
                let p_sub_dir = root.join("sub");
                let p_sub_exe = p_sub_dir.join("sub.exe");
                create_file(&p_root_exe);
                create_dir(&p_sub_dir);
                create_file(&p_sub_exe);
                let expected: HashSet<(String, CandidateKind)> = vec![
                    (p_root_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                    (p_sub_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                ].into_iter().collect();
                (tmp, vec![root], None, expected)
            },
        },
        Case { // 複数 root の集約
            name: "複数ルートを集約する",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let parent = tmp.path();
                let root_a = parent.join("A");
                let root_b = parent.join("B");
                create_dir(&root_a);
                create_dir(&root_b);
                let a_exe = root_a.join("a.exe");
                let b_lnk = root_b.join("b.lnk");
                let b_sub = root_b.join("sub");
                create_file(&a_exe);
                create_file(&b_lnk);
                create_dir(&b_sub);

                let expected: HashSet<(String, CandidateKind)> = vec![
                    (a_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                    (b_lnk.to_string_lossy().to_string(), CandidateKind::Shortcut),
                ].into_iter().collect();
                (tmp, vec![root_a, root_b], None, expected)
            },
        },
        Case { // 除外（ファイルとディレクトリ）
            name: "除外キャッシュでファイルとディレクトリを除外できる",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let p_exe = root.join("game.exe");
                let p_lnk = root.join("shortcut.lnk");
                let p_dir = root.join("folder");
                create_file(&p_exe);
                create_file(&p_lnk);
                create_dir(&p_dir);
                let mut exclude: ExploredCache = ExploredCache::default();
                exclude.insert(p_exe.to_string_lossy().to_string());
                exclude.insert(p_dir.to_string_lossy().to_string());
                let expected: HashSet<(String, CandidateKind)> = vec![
                    (p_lnk.to_string_lossy().to_string(), CandidateKind::Shortcut),
                ].into_iter().collect();
                (tmp, vec![root], Some(exclude), expected)
            },
        },
        Case { // 除外（存在しないパスは無視）
            name: "除外キャッシュに存在しないパスは影響しない",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let p_exe = root.join("game.exe");
                create_file(&p_exe);
                let mut exclude: ExploredCache = ExploredCache::default();
                exclude.insert(root.join("nonexistent.exe").to_string_lossy().to_string());
                let expected: HashSet<(String, CandidateKind)> = vec![
                    (p_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                ].into_iter().collect();
                (tmp, vec![root], Some(exclude), expected)
            },
        },
        Case { // 不存在 root を無視
            name: "存在しないルートは無視される",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let missing = root.join("missing-not-exist");
                let p_dir = root.join("dir");
                create_dir(&p_dir);
                let expected: HashSet<(String, CandidateKind)> = vec![
                ].into_iter().collect();
                (tmp, vec![missing, root], None, expected)
            },
        },
        Case { // 拡張子の大文字小文字無視
            name: "拡張子判定は大文字小文字を無視する",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let upper_exe = root.join("UPPER.EXE");
                let mixed_lnk = root.join("MiXeD.LnK");
                create_file(&upper_exe);
                create_file(&mixed_lnk);
                let expected: HashSet<(String, CandidateKind)> = vec![
                    (upper_exe.to_string_lossy().to_string(), CandidateKind::Exe),
                    (mixed_lnk.to_string_lossy().to_string(), CandidateKind::Shortcut),
                ].into_iter().collect();
                (tmp, vec![root], None, expected)
            },
        },
        Case { // ディレクトリ名が .exe で終わる場合は Folder として扱う
            name: "拡張子風のディレクトリはFolderとして扱われる",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let root = tmp.path().to_path_buf();
                let dir_like_exe = root.join("folder.exe");
                create_dir(&dir_like_exe);
                let expected: HashSet<(String, CandidateKind)> = vec![
                ].into_iter().collect();
                (tmp, vec![root], None, expected)
            },
        },
    ];

    // Act & Assert
    let fs_impl = LocalFileSystem::default();
    for c in cases {
        let (tmp, roots, exclude, expected) = (c.build)(); // Arrange (per-case)
        let actual = collect_set(&fs_impl, &roots, exclude); // Act
        assert_eq!(actual, expected, "case: {}", c.name); // Assert
        drop(tmp);
    }
}

#[test]
fn stat_テーブルテスト() {
    // Arrange
    struct Case {
        name: &'static str,
        build: fn() -> (tempfile::TempDir, PathBuf, bool), // bool: expect_ok
    }

    let cases: Vec<Case> = vec![
        Case {
            name: "既存ファイルはokを返す",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let p = tmp.path().join("f.bin");
                create_file(&p);
                (tmp, p, true)
            },
        },
        Case {
            name: "存在しないパスはerrを返す",
            build: || {
                let tmp = tempfile::tempdir().unwrap();
                let p = tmp.path().join("missing.file");
                (tmp, p, false)
            },
        },
    ];

    let fs_impl = LocalFileSystem::default();
    for c in cases {
        let (tmp, path, expect_ok) = (c.build)(); // Arrange (per-case)
        let result = fs_impl.stat(&path); // Act
        if expect_ok {
            let meta = result.expect(c.name);
            assert!(meta.is_file(), "case: {}", c.name);
        } else {
            assert!(result.is_err(), "case: {}", c.name);
        }
        drop(tmp);
    }
}


