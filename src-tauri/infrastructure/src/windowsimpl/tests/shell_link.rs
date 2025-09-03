use std::path::{Path, PathBuf};

use crate::windowsimpl::shell_link::ShellLinkImpl;
use domain::windows::shell_link::{ShellLink, CreateShortcutRequest};

fn resolve_hello_world_exe() -> PathBuf {
    // canonicalize() は \\?\ プレフィックスを付ける可能性があるため避ける
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest_dir.join("..").join("bin").join("hello_world.exe");
    assert!(p.exists(), "hello_world.exe が見つかりません: {}", p.display());
    p
}

fn normalize_lower(p: &Path) -> String {
    let c = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let mut s = c.display().to_string();
    if s.starts_with("\\\\?\\") {
        s = s.trim_start_matches("\\\\?\\").to_string();
    }
    s.to_lowercase()
}

#[test]
fn ショートカットの作成と実行ができる() {
    // Arrange
    let exe = resolve_hello_world_exe();
    assert!(exe.exists(), "hello_world.exe が存在しません: {}", exe.display());

    // バイナリと同じディレクトリに配置
    let bin_dir = exe.parent().expect("親ディレクトリが取得できません");
    let link_path = bin_dir.join("hello_world_test.lnk");

    // 事前掃除
    let _ = std::fs::remove_file(&link_path);

    let shell_link = ShellLinkImpl::new();
    let exe_str = exe.to_string_lossy();

    // Act: .lnk を作成（WD/引数/アイコンは省略）
    shell_link
        .create_bulk(vec![CreateShortcutRequest {
            target_path: exe_str.to_string(),
            dest_lnk_path: link_path.display().to_string(),
            working_dir: None,
            arguments: None,
            icon_path: None,
        }])
        .expect("lnk 作成に失敗しました");

    // Assert: .lnk が存在
    assert!(link_path.exists(), ".lnk が作成されていません: {}", link_path.display());

    // さらなる検証: メタデータからターゲットパスが取得できる
    let metas = shell_link
        .get_lnk_metadatas(vec![link_path.display().to_string()])
        .expect("lnk メタデータ取得に失敗");
    let meta = metas
        .get(&link_path.display().to_string())
        .expect("メタデータに .lnk が含まれていません");

    let meta_norm = normalize_lower(Path::new(&meta.path));
    let exe_norm = normalize_lower(&exe);
    assert_eq!(
        meta_norm,
        exe_norm,
        "メタデータのターゲットパスが一致しません: {} vs {}",
        meta.path,
        exe.display()
    );

    // .lnk を実行して終了コード 0 を確認（ドメイン経由のメソッドを使用）
    let code = shell_link.execute_lnk(&link_path.display().to_string(), false).expect(".lnk 実行に失敗");
    assert_eq!(code, Some(0), ".lnk 実行の終了コードが 0 ではありません: {:?}", code);
}

#[test]
fn アイコンとワーキングディレクトリを設定して作成できメタが取得できる() {
    // Arrange
    let exe = resolve_hello_world_exe();
    assert!(exe.exists(), "hello_world.exe が存在しません: {}", exe.display());

    let bin_dir = exe.parent().expect("親ディレクトリが取得できません");
    let link_path = bin_dir.join("hello_world_icon_wd_test.lnk");
    let icon_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("icons")
        .join("icon.ico");
    assert!(icon_path.exists(), "icon.ico が見つかりません: {}", icon_path.display());

    // 事前掃除
    let _ = std::fs::remove_file(&link_path);

    let shell_link = ShellLinkImpl::new();
    let exe_str = exe.to_string_lossy();

    // Act
    shell_link
        .create_bulk(vec![CreateShortcutRequest {
            target_path: exe_str.to_string(),
            dest_lnk_path: link_path.display().to_string(),
            working_dir: Some(bin_dir.display().to_string()),
            arguments: None,
            icon_path: Some(icon_path.display().to_string()),
        }])
        .expect("lnk 作成に失敗しました");

    // Assert: .lnk が存在
    assert!(link_path.exists(), ".lnk が作成されていません: {}", link_path.display());

    // メタデータ検証（ターゲットパスとアイコンパス）
    let metas = shell_link
        .get_lnk_metadatas(vec![link_path.display().to_string()])
        .expect("lnk メタデータ取得に失敗");
    let meta = metas
        .get(&link_path.display().to_string())
        .expect("メタデータに .lnk が含まれていません");

    let meta_norm = normalize_lower(Path::new(&meta.path));
    let exe_norm = normalize_lower(&exe);
    assert_eq!(meta_norm, exe_norm, "ターゲットパスが一致しません: {} vs {}", meta.path, exe.display());

    let meta_icon_norm = normalize_lower(Path::new(&meta.icon));
    let icon_norm = normalize_lower(&icon_path);
    assert_eq!(meta_icon_norm, icon_norm, "アイコンパスが一致しません: {} vs {}", meta.icon, icon_path.display());
}
