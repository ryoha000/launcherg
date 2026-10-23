use std::{path::PathBuf, sync::OnceLock};

pub fn ensure_extract_icon_sidecar() -> PathBuf {
    static SIDECAR: OnceLock<PathBuf> = OnceLock::new();
    SIDECAR
        .get_or_init(|| {
            let path = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("extract-icon.exe");
            std::fs::write(
                &path,
                include_bytes!("../../bin/extract-icon-x86_64-pc-windows-msvc.exe"),
            )
            .unwrap();
            path
        })
        .clone()
}

pub fn icon_source_exe() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let source = dir.join("icon-fixture.cs");
        let target = dir.join("icon-fixture.exe");
        std::fs::write(&source, "public class Fixture { public static void Main() {} }").unwrap();
        let compiler = PathBuf::from(std::env::var_os("WINDIR").unwrap())
            .join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
        let icon = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../icons/icon.ico");
        let output = std::process::Command::new(compiler).args([
            "/nologo", "/target:winexe", &format!("/win32icon:{}", icon.display()),
            &format!("/out:{}", target.display()), &source.to_string_lossy(),
        ]).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
        target
    }).clone()
}
