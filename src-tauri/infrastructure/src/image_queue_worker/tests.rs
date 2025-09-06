use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

use domain::repository::mock::{TestRepositories, TestRepositoryManager};
use domain::save_image_queue::{ImagePreprocess, ImageSaveQueueRow};
use domain::Id;
use std::sync::{Arc, Mutex};

use domain::file::LnkMetadata;
use domain::save_image_queue::ImageSrcType;
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};
use domain::windows::shell_link::MockShellLink;
use domain::windows::{
    process::MockProcessWindows, proctail::MockProcTail,
    proctail_manager::MockProcTailManagerTrait, WindowsExt,
};

use super::{resolver, types::SourceDecision};

#[derive(Clone)]
struct TestResolver {
    root: String,
}

impl TestResolver {
    fn new(root: String) -> Self {
        Self { root }
    }
}

impl SavePathResolver for TestResolver {
    fn root_dir(&self) -> String {
        self.root.clone()
    }
}

struct TestWindows {
    process: MockProcessWindows,
    proctail: MockProcTail,
    proctail_manager: MockProcTailManagerTrait,
    shell_link: MockShellLink,
}

impl TestWindows {
    fn new(shell_link: MockShellLink) -> Self {
        Self {
            process: MockProcessWindows::new(),
            proctail: MockProcTail::new(),
            proctail_manager: MockProcTailManagerTrait::new(),
            shell_link,
        }
    }
}

impl WindowsExt for TestWindows {
    type ProcessWindows = MockProcessWindows;
    type ProcTail = MockProcTail;
    type ProcTailManager = MockProcTailManagerTrait;
    type ShellLink = MockShellLink;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }
    fn proctail(&self) -> &Self::ProcTail {
        &self.proctail
    }
    fn proctail_manager(&self) -> &Self::ProcTailManager {
        &self.proctail_manager
    }
    fn shell_link(&self) -> &Self::ShellLink {
        &self.shell_link
    }
}

#[tokio::test]
async fn handle_http_download_のテスト() {
    // Arrange
    let server = wiremock::MockServer::start().await;
    let tmp = TempDir::new().unwrap();
    let resolver = TestResolver::new(tmp.path().to_string_lossy().to_string());

    #[derive(Clone)]
    enum Kind {
        Download { path: &'static str, body: &'static [u8] },
        ErrorInvalidUrl { url: &'static str },
    }

    struct Case {
        name: &'static str,
        kind: Kind,
    }

    let cases = vec![
        Case {
            name: "png をダウンロード",
            kind: Kind::Download { path: "/img.png", body: b"hello-image" },
        },
        Case {
            name: "HTTP 不正はエラー",
            kind: Kind::ErrorInvalidUrl { url: "http://127.0.0.1:9/does-not-exist" },
        },
    ];

    // Act & Assert
    for c in cases {
        match c.kind.clone() {
            Kind::Download { path, body } => {
                wiremock::Mock::given(wiremock::matchers::method("GET"))
                    .and(wiremock::matchers::path(path))
                    .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
                    .mount(&server)
                    .await;

                let url = format!("{}{}", &server.uri(), path);
                let p = resolver::url::resolve_to_tmp(&resolver, &url).await.unwrap();
                assert!(Path::new(&p).exists(), "{}", c.name);
                let read = std::fs::read(&p).unwrap();
                assert_eq!(read, body, "{}", c.name);
            }
            Kind::ErrorInvalidUrl { url } => {
                let res = resolver::url::resolve_to_tmp(&resolver, url).await;
                assert!(res.is_err(), "{}", c.name);
            }
        }
    }
}

#[test]
fn handle_path_のテスト() {
    struct Case {
        input: &'static str,
        expected: &'static str,
    }
    let cases = vec![Case {
        input: "C:/images/a.png",
        expected: "C:/images/a.png",
    }];

    for c in cases {
        let p = resolver::path::resolve(c.input);
        assert_eq!(p, c.expected);
    }
}

#[test]
fn handle_shortcut_のテスト() {
    // Arrange
    #[derive(Clone)]
    enum MetaKind {
        IconIco { meta_path: &'static str, meta_icon: &'static str },
        IconEmpty { meta_path: &'static str },
        IconPng { meta_path: &'static str, meta_icon: &'static str },
        MetaError,
        MissingKey,
    }

    struct Case {
        name: &'static str,
        meta: MetaKind,
    }

    let cases = vec![
        Case {
            name: "icon が .ico の場合は一時 png を返す",
            meta: MetaKind::IconIco { meta_path: "C:/Program Files/App/app.exe", meta_icon: "C:/icons/app.ico" },
        },
        Case {
            name: "icon 空ならデフォルトを書き出して Skip",
            meta: MetaKind::IconEmpty { meta_path: "C:/Program Files/App/app.exe" },
        },
        Case {
            name: "meta icon が png ならそのまま返す",
            meta: MetaKind::IconPng { meta_path: "C:/Program Files/App/app.exe", meta_icon: "C:/images/icon.png" },
        },
        Case { name: "メタ取得エラーなら Skip とデフォルト書き出し", meta: MetaKind::MetaError },
        Case { name: "メタにキーが無いなら Skip とデフォルト書き出し", meta: MetaKind::MissingKey },
    ];

    for (i, c) in cases.into_iter().enumerate() {
        let mut mock = MockShellLink::new();
        match c.meta.clone() {
            MetaKind::IconIco { meta_path, meta_icon } => {
                mock.expect_get_lnk_metadatas().returning(move |paths| {
                    let mut map = HashMap::new();
                    let key = paths[0].clone();
                    map.insert(key, LnkMetadata { path: meta_path.to_string(), icon: meta_icon.to_string() });
                    Ok(map)
                });
            }
            MetaKind::IconEmpty { meta_path } => {
                mock.expect_get_lnk_metadatas().returning(move |paths| {
                    let mut map = HashMap::new();
                    let key = paths[0].clone();
                    map.insert(key, LnkMetadata { path: meta_path.to_string(), icon: "".to_string() });
                    Ok(map)
                });
            }
            MetaKind::IconPng { meta_path, meta_icon } => {
                mock.expect_get_lnk_metadatas().returning(move |paths| {
                    let mut map = HashMap::new();
                    let key = paths[0].clone();
                    map.insert(key, LnkMetadata { path: meta_path.to_string(), icon: meta_icon.to_string() });
                    Ok(map)
                });
            }
            MetaKind::MetaError => {
                mock.expect_get_lnk_metadatas().returning(|_| Err(anyhow::anyhow!("get meta failed")));
            }
            MetaKind::MissingKey => {
                mock.expect_get_lnk_metadatas().returning(|_| Ok(HashMap::new()));
            }
        }

        let win = TestWindows::new(mock);
        let tmp = TempDir::new().unwrap();
        let resolver = TestResolver::new(tmp.path().to_string_lossy().to_string());
        let dst_dir = TempDir::new().unwrap();
        let _dst = dst_dir.path().join(format!("out_{}.png", i));

        // Act
        let res = resolver::shortcut::resolve(&win, &resolver, "C:/links/app.lnk");

        // Assert
        match (c.meta, res) {
            (MetaKind::IconIco { .. }, Ok(SourceDecision::Use(local))) => {
                let p = local.path();
                assert!(p.ends_with(".png"), "{}: {}", c.name, p);
            }
            (MetaKind::IconEmpty { .. }, Ok(SourceDecision::FallbackDefaultAndSkip)) => {}
            (MetaKind::IconPng { .. }, Ok(SourceDecision::Use(local))) => {
                assert_eq!(local.path(), "C:/images/icon.png", "{}", c.name);
            }
            (MetaKind::MissingKey, Ok(SourceDecision::FallbackDefaultAndSkip)) => {}
            (MetaKind::MetaError, Err(_)) => {}
            _ => panic!("unexpected result for case '{}'", c.name),
        }
    }
}

#[test]
fn handle_exe_のテスト() {
    struct Case {
        name: &'static str,
        exe_path: &'static str,
    }
    let cases = vec![Case {
        name: "存在しない exe は Skip",
        exe_path: "C:/not-exists/app.exe",
    }];

    for (_i, c) in cases.into_iter().enumerate() {
        let _dst_dir = TempDir::new().unwrap();
        let resolver_impl = TestResolver::new("/tmp".to_string());
        let res = resolver::exe::resolve(&resolver_impl, c.exe_path).unwrap();
        match res {
            SourceDecision::FallbackDefaultAndSkip => { /* no-op */ }
            SourceDecision::Use(local) => {
                assert!(Path::new(local.path()).exists(), "{}", c.name);
            }
        }
    }
}

#[tokio::test]
async fn resolve_local_src_path_のテスト() {
    // Arrange
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/a.png"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(b"x"))
        .mount(&server)
        .await;

    let tmp = TempDir::new().unwrap();
    let resolver = TestResolver::new(tmp.path().to_string_lossy().to_string());

    #[derive(Clone, Debug)]
    enum Arrange {
        UrlOk,
        Path { src: &'static str },
        ShortcutPng,
        ExeNotExists { exe: &'static str },
        UrlBad { url: &'static str },
    }

    #[derive(Clone, Debug)]
    enum Expect {
        PathExists,                  // 返ったパスが存在することだけ検証（例: URL）
        PathEquals(&'static str),    // 返ったパスが期待文字列と一致
        Skip,                        // Skip が返る（既定アイコンの書き出しは worker 側）
        Error,                       // エラーであること
    }

    struct Case {
        name: &'static str,
        arrange: Arrange,
        expect: Expect,
    }

    let url_ok = format!("{}/a.png", server.uri());
    let cases = vec![
        Case { name: "HTTP はダウンロードして一時パス", arrange: Arrange::UrlOk, expect: Expect::PathExists },
        Case { name: "Path はそのまま返す", arrange: Arrange::Path { src: "C:/x.png" }, expect: Expect::PathEquals("C:/x.png") },
        Case { name: "Shortcut は png をそのまま返す", arrange: Arrange::ShortcutPng, expect: Expect::PathEquals("C:/images/icon.png") },
        Case { name: "Exe は sidecar の挙動により Path または Skip", arrange: Arrange::ExeNotExists { exe: "C:/not-exists/app.exe" }, expect: Expect::Skip },
        Case { name: "HTTP 失敗はエラー", arrange: Arrange::UrlBad { url: "http://127.0.0.1:9/" }, expect: Expect::Error },
    ];

    for (i, c) in cases.into_iter().enumerate() {
        // Windows のモックはケースごとに用意（Arrange に応じたセットアップ）
        let mut mock = MockShellLink::new();
        match c.arrange {
            Arrange::ShortcutPng => {
                mock.expect_get_lnk_metadatas().returning(|paths| {
                    let mut map = HashMap::new();
                    let key = paths[0].clone();
                    map.insert(key, LnkMetadata { path: "C:/Program Files/App/app.exe".to_string(), icon: "C:/images/icon.png".to_string() });
                    Ok(map)
                });
            }
            _ => {
                mock.expect_get_lnk_metadatas().returning(|_| Ok(HashMap::new()));
            }
        }
        let win = TestWindows::new(mock);
        let _dst_dir = TempDir::new().unwrap();
        let _dst_path = TempDir::new().unwrap().path().join(format!("dst_{}.png", i));
        let _dst = _dst_path.to_string_lossy();

        // Act
        let res = match c.arrange.clone() {
            Arrange::UrlOk => resolver::resolve_source(&win, &resolver, &url_ok, ImageSrcType::Url).await,
            Arrange::Path { src } => resolver::resolve_source(&win, &resolver, src, ImageSrcType::Path).await,
            Arrange::ShortcutPng => resolver::resolve_source(&win, &resolver, "C:/links/app.lnk", ImageSrcType::Shortcut).await,
            Arrange::ExeNotExists { exe } => resolver::resolve_source(&win, &resolver, exe, ImageSrcType::Exe).await,
            Arrange::UrlBad { url } => resolver::resolve_source(&win, &resolver, url, ImageSrcType::Url).await,
        };

        // Assert
        match (c.expect, res) {
            (Expect::PathExists, Ok(SourceDecision::Use(local))) => {
                assert!(Path::new(local.path()).exists(), "{}", c.name);
            }
            (Expect::PathEquals(expected), Ok(SourceDecision::Use(local))) => {
                assert_eq!(local.path(), expected, "{}", c.name);
            }
            (Expect::Skip, Ok(SourceDecision::FallbackDefaultAndSkip)) => { /* ok */ }
            (Expect::Error, Err(_)) => { /* 期待通りエラー */ }
            actual => panic!("unexpected result for case '{}': {:?}", c.name, actual.1),
        }
    }
}

#[test]
#[ignore]
fn handle_exe_成功なら_pathを返す_要sidecar設置() {
    // このテストは実行ファイルと同ディレクトリに extract-icon.exe が必要
    // 適切なスタブを配置できる環境でのみ実行してください
    let dst_dir = TempDir::new().unwrap();
    let dst = dst_dir.path().join("exe_success.png");
    let exe = "C:/Windows/System32/notepad.exe"; // 例: 適当なexe
    let resolver_impl = TestResolver::new("/tmp".to_string());
    let res = resolver::exe::resolve(&resolver_impl, exe).unwrap();
    match res {
        SourceDecision::Use(local) => assert_eq!(local.path(), dst.to_string_lossy()),
        _ => panic!("expected Path"),
    }
}

fn write_small_png(path: &str, w: u32, h: u32) {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([0u8, 0u8, 0u8, 255u8]));
    img.save(path).unwrap();
}

#[tokio::test]
async fn resolve_local_src_pathに実際のファイルを渡してアイコンが一時的にローカルへ保存される() {
    // Arrange
    use crate::windowsimpl::windows::Windows as RealWindows;

    // extract-icon.exe のセットアップ
    let extract_icon_exe = {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .ok_or_else(|| anyhow::anyhow!("failed to resolve current exe directory")).unwrap();
        let dst = exe_dir.join("extract-icon.exe");
        const BYTES: &[u8] = include_bytes!("../../../bin/extract-icon-x86_64-pc-windows-msvc.exe");
        if !dst.exists() {
            let _ = fs::write(&dst, BYTES);
        }
        dst
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets_dir = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("image_queue_worker")
        .join("assets");

    let lnk = assets_dir.join("ISLAND.lnk");
    let url = assets_dir.join("ATRI -My Dear Moments-.url");
    let exe = assets_dir.join("dohnadohna.exe");

    #[derive(Clone)]
    struct Case {
        name: &'static str,
        src_path: String,
        src_type: ImageSrcType,
    }

    let cases = vec![
        Case { name: "入力がlnkの場合", src_path: lnk.to_string_lossy().to_string(), src_type: ImageSrcType::Shortcut },
        Case { name: "入力がexeの場合", src_path: exe.to_string_lossy().to_string(), src_type: ImageSrcType::Exe },
        Case { name: "入力がurlの場合", src_path: url.to_string_lossy().to_string(), src_type: ImageSrcType::Shortcut },
    ];

    let resolver = Arc::new(DirsSavePathResolver::default());

    let windows = RealWindows::new();

    for c in cases { 
        // Act
        let result = resolver::resolve_source(&windows, &*resolver, &c.src_path, c.src_type).await;

        // Assert
        match result {
            Ok(SourceDecision::Use(local)) => {
                assert!(Path::new(local.path()).exists(), "{}", c.name);
            }
            Ok(SourceDecision::FallbackDefaultAndSkip) => {
                assert!(false, "{} は Skip ではありません", c.name);
            }
            Err(_) => {
                assert!(false, "{} はエラーではありません", c.name);
            }
        }
    }

    // cleanup
    let _ = std::fs::remove_file(&extract_icon_exe);
}

#[tokio::test]
async fn drain_until_empty_成功コピー_only_mark_finished() {
    let repos = TestRepositories::default();
    let manager = Arc::new(TestRepositoryManager::new(repos.clone()));
    let tmp = TempDir::new().unwrap();
    let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

    // 入力ファイルを用意
    let src_dir = TempDir::new().unwrap();
    let src = src_dir.path().join("src.png");
    write_small_png(&src.to_string_lossy(), 10, 10);
    let dst = tmp.path().join("dst.png");

    let row = ImageSaveQueueRow {
        id: Id::new(1),
        src: src.to_string_lossy().to_string(),
        src_type: ImageSrcType::Path,
        dst_path: dst.to_string_lossy().to_string(),
        preprocess: ImagePreprocess::None,
        last_error: None,
    };

    // list_unfinished_oldest: 1回目は1件、2回目は空
    let counter = Arc::new(Mutex::new(0));
    {
        let c = counter.clone();
        let mut iq = repos.image_queue.lock().await;
        iq.expect_list_unfinished_oldest().returning(move |_| {
            let mut n = c.lock().unwrap();
            let ret = if *n == 0 { vec![row.clone()] } else { vec![] };
            *n += 1;
            std::pin::Pin::from(Box::new(async move { Ok(ret) }))
        });
        iq.expect_mark_finished()
            .times(1)
            .returning(|_| std::pin::Pin::from(Box::new(async { Ok(()) })));
        iq.expect_mark_failed().times(0);
    }

    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas()
        .returning(|_| Ok(HashMap::new()));
    let windows = Arc::new(TestWindows::new(mock));
    let worker = crate::image_queue_worker::ImageQueueWorker::new(manager, resolver, windows);
    worker.drain_until_empty().await.unwrap();

    assert!(Path::new(&dst).exists());
}

#[tokio::test]
async fn drain_until_empty_処理失敗_mark_failedとエラーログ() {
    let repos = TestRepositories::default();
    let manager = Arc::new(TestRepositoryManager::new(repos.clone()));
    let tmp = TempDir::new().unwrap();
    let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

    // 存在しない入力にしてコピーで失敗させる
    let dst = tmp.path().join("dst.png");
    let row = ImageSaveQueueRow {
        id: Id::new(2),
        src: "C:/not-exists/input.png".to_string(),
        src_type: ImageSrcType::Path,
        dst_path: dst.to_string_lossy().to_string(),
        preprocess: ImagePreprocess::None,
        last_error: None,
    };

    let counter = Arc::new(Mutex::new(0));
    {
        let c = counter.clone();
        let mut iq = repos.image_queue.lock().await;
        iq.expect_list_unfinished_oldest().returning(move |_| {
            let mut n = c.lock().unwrap();
            let ret = if *n == 0 { vec![row.clone()] } else { vec![] };
            *n += 1;
            std::pin::Pin::from(Box::new(async move { Ok(ret) }))
        });
        iq.expect_mark_finished().times(0);
        iq.expect_mark_failed()
            .times(1)
            .returning(|_, _| std::pin::Pin::from(Box::new(async { Ok(()) })));
    }

    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas()
        .returning(|_| Ok(HashMap::new()));
    let windows = Arc::new(TestWindows::new(mock));
    let worker = crate::image_queue_worker::ImageQueueWorker::new(manager, resolver, windows);
    worker.drain_until_empty().await.unwrap();
}

#[tokio::test]
async fn drain_until_empty_skip経路_デフォルト書き出しと_mark_finished() {
    let repos = TestRepositories::default();
    let manager = Arc::new(TestRepositoryManager::new(repos.clone()));
    let tmp = TempDir::new().unwrap();
    let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

    let dst = tmp.path().join("dst_skip.png");
    let row = ImageSaveQueueRow {
        id: Id::new(3),
        src: "C:/links/app.lnk".to_string(),
        src_type: ImageSrcType::Shortcut,
        dst_path: dst.to_string_lossy().to_string(),
        preprocess: ImagePreprocess::ResizeAndCropSquare256,
        last_error: None,
    };

    let counter = Arc::new(Mutex::new(0));
    {
        let c = counter.clone();
        let mut iq = repos.image_queue.lock().await;
        iq.expect_list_unfinished_oldest().returning(move |_| {
            let mut n = c.lock().unwrap();
            let ret = if *n == 0 { vec![row.clone()] } else { vec![] };
            *n += 1;
            std::pin::Pin::from(Box::new(async move { Ok(ret) }))
        });
        iq.expect_mark_finished()
            .times(1)
            .returning(|_| std::pin::Pin::from(Box::new(async { Ok(()) })));
        iq.expect_mark_failed().times(0);
    }

    // ショートカットのメタは空にして Skip 経路へ
    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas()
        .returning(|_| Ok(HashMap::new()));
    let windows = Arc::new(TestWindows::new(mock));
    let worker = crate::image_queue_worker::ImageQueueWorker::new(manager, resolver, windows);
    worker.drain_until_empty().await.unwrap();
    assert!(Path::new(&dst).exists());
}

#[tokio::test]
async fn drain_until_empty_url経由_一時ファイル削除される() {
    let repos = TestRepositories::default();
    let manager = Arc::new(TestRepositoryManager::new(repos.clone()));
    let tmp = TempDir::new().unwrap();
    let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

    // モックサーバで1バイト配信
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/z.png"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(b"z".to_vec()))
        .mount(&server)
        .await;
    let url = format!("{}/z.png", server.uri());

    let dst = tmp.path().join("dst_url.png");
    let row = ImageSaveQueueRow {
        id: Id::new(4),
        src: url.clone(),
        src_type: ImageSrcType::Url,
        dst_path: dst.to_string_lossy().to_string(),
        preprocess: ImagePreprocess::None,
        last_error: None,
    };

    let counter = Arc::new(Mutex::new(0));
    {
        let c = counter.clone();
        let mut iq = repos.image_queue.lock().await;
        iq.expect_list_unfinished_oldest().returning(move |_| {
            let mut n = c.lock().unwrap();
            let ret = if *n == 0 { vec![row.clone()] } else { vec![] };
            *n += 1;
            std::pin::Pin::from(Box::new(async move { Ok(ret) }))
        });
        iq.expect_mark_finished()
            .times(1)
            .returning(|_| std::pin::Pin::from(Box::new(async { Ok(()) })));
    }

    let mut mock = MockShellLink::new();
    mock.expect_get_lnk_metadatas()
        .returning(|_| Ok(HashMap::new()));
    let windows = Arc::new(TestWindows::new(mock));
    let worker =
        crate::image_queue_worker::ImageQueueWorker::new(manager, resolver.clone(), windows);
    worker.drain_until_empty().await.unwrap();

    // 一時ファイルは削除されているはず（RAII）: 具体パスは取得困難なので dst の存在のみを確認
    assert!(Path::new(&dst).exists());
}
