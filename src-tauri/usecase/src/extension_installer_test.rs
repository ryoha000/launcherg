use super::*;

struct Config {
    resources: PathBuf,
    data: PathBuf,
}
impl AppConfigProvider for Config {
    fn get_app_config_dir(&self) -> String {
        self.data.to_string_lossy().into()
    }
    fn get_resource_dir(&self) -> Result<PathBuf, io::Error> {
        Ok(self.resources.clone())
    }
}

#[tokio::test]
async fn 配布物だけで拡張を展開し公開鍵から固定のidを取得できる() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("installed resources");
    let source = resources.join("integrations/browser-extension");
    fs::create_dir_all(source.join("background")).unwrap();
    fs::write(
        source.join("manifest.json"),
        include_str!("../../../browser-extension/manifest.json"),
    )
    .unwrap();
    fs::write(source.join("background/background.js"), b"test").unwrap();
    let installer = ExtensionInstallerUseCase::new(Arc::new(Config {
        resources,
        data: temp.path().join("user data"),
    }));
    let info = installer.get_extension_manifest_info().await.unwrap();
    assert_eq!(info.extension_id.len(), 32);
    assert!(info
        .extension_id
        .bytes()
        .all(|b| (b'a'..=b'p').contains(&b)));
    let package = installer.generate_extension_package().await.unwrap();
    let mut zip = zip::ZipArchive::new(fs::File::open(&package.package_path).unwrap()).unwrap();
    assert!(zip.by_name("background/background.js").is_ok());
    let destination = installer.copy_extension_for_development().await.unwrap();
    assert_eq!(
        fs::read(Path::new(&destination).join("background/background.js")).unwrap(),
        b"test"
    );
    // Invalid IDs cannot reach registry writes or launch an external command.
    assert!(installer
        .setup_native_messaging_host(Some("invalid-id".into()))
        .await
        .is_err());
}

#[test]
fn 公開鍵のidはchromeのsha256アルゴリズムと一致する() {
    // SHA256("abc") starts with ba7816bf8f01cfea414140de5dae2223.
    assert_eq!(
        extension_id_from_key("YWJj").unwrap(),
        "lkhibglpipabmpokebebeanofnkocccd"
    );
    assert!(extension_id_from_key("?").is_err());
    assert!(extension_id_from_key("").is_err());
}
