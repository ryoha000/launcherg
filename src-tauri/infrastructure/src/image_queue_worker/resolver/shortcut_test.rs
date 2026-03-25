use std::collections::HashMap;

use domain::file::LnkMetadata;
use domain::service::save_path_resolver::DirsSavePathResolver;
use domain::windows::shell_link::MockShellLink;
use domain::windows::{process::MockProcessWindows, WindowsExt};

use super::shortcut::resolve;

struct TestWindows {
    process: MockProcessWindows,
    shell_link: MockShellLink,
}
impl TestWindows {
    fn new(shell_link: MockShellLink) -> Self {
        Self {
            process: MockProcessWindows::new(),
            shell_link,
        }
    }
}
impl WindowsExt for TestWindows {
    type ProcessWindows = MockProcessWindows;
    type ShellLink = MockShellLink;
    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }
    fn shell_link(&self) -> &Self::ShellLink {
        &self.shell_link
    }
}

#[test]
fn shortcut_メタにicoなら一時png() {
    let mut mock = MockShellLink::new();
    let ico_path = {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        std::path::Path::new(&manifest_dir)
            .join("..")
            .join("icons")
            .join("icon.ico")
            .to_string_lossy()
            .to_string()
    };
    mock.expect_get_lnk_metadatas().returning(move |paths| {
        let mut map = HashMap::new();
        let key = paths[0].clone();
        map.insert(
            key,
            LnkMetadata {
                path: "C:/app/app.exe".into(),
                icon: ico_path.clone(),
            },
        );
        Ok(map)
    });
    let win = TestWindows::new(mock);
    let resolver = DirsSavePathResolver::default();
    let res = resolve(&win, &resolver, "C:/links/app.lnk");
    match res {
        Ok(super::super::types::SourceDecision::Use(local)) => {
            assert!(local.path().ends_with(".png"));
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn shortcut_メタpngならそのまま() {
    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas().returning(|paths| {
        let mut map = HashMap::new();
        let key = paths[0].clone();
        map.insert(
            key,
            LnkMetadata {
                path: "C:/app/app.exe".into(),
                icon: "C:/images/icon.png".into(),
            },
        );
        Ok(map)
    });
    let win = TestWindows::new(mock);
    let resolver = DirsSavePathResolver::default();
    let res = resolve(&win, &resolver, "C:/links/app.lnk");
    match res {
        Ok(super::super::types::SourceDecision::FallbackDefaultAndSkip { .. }) => {}
        _ => panic!("unexpected"),
    }
}

#[test]
fn shortcut_情報なしはskip() {
    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas()
        .returning(|_| Ok(HashMap::new()));
    let win = TestWindows::new(mock);
    let resolver = DirsSavePathResolver::default();
    let res = resolve(&win, &resolver, "C:/links/app.lnk");
    match res {
        Ok(super::super::types::SourceDecision::FallbackDefaultAndSkip { .. }) => {}
        _ => panic!("unexpected"),
    }
}
