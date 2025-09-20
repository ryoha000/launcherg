use std::path::PathBuf;

pub trait ExtractIconRunner: Send + Sync {
    /// 指定 exe から PNG を抽出
    /// 戻り値: 成功したら true（dst に出力済み）、失敗なら false
    fn extract_icon(&self, width: u32, exe_path: &str, dst_path: &str) -> anyhow::Result<bool>;
}

pub struct ExtractIconRunnerImpl {
    sidecar_path: PathBuf,
}

impl ExtractIconRunnerImpl {
    fn resolve_extract_icon_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|d| d.join("extract-icon.exe"))
            .unwrap_or_else(|| PathBuf::from("extract-icon.exe"))
    }

    pub fn new() -> Self {
        Self {
            sidecar_path: Self::resolve_extract_icon_path(),
        }
    }

    pub fn new_with_sidecar_path<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            sidecar_path: path.into(),
        }
    }
}

impl ExtractIconRunner for ExtractIconRunnerImpl {
    fn extract_icon(&self, width: u32, exe_path: &str, dst_path: &str) -> anyhow::Result<bool> {
        let sidecar = &self.sidecar_path;
        if !sidecar.exists() || !std::path::Path::new(exe_path).exists() {
            return Ok(false);
        }
        let output = std::process::Command::new(sidecar)
            .args([&width.to_string(), exe_path, dst_path])
            .output();
        match output {
            Ok(o) if o.status.success() => Ok(true),
            _ => Ok(false),
        }
    }
}
