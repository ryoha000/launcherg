use base64::Engine;
use derive_new::new;
use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Manager};
#[cfg(windows)]
use winreg::{enums::*, RegKey};

const HOST_NAME: &str = "moe.ryoha.launcherg.extension_host";
const REGISTRY_KEYS: [(&str, &str); 2] = [
    (
        "Chrome",
        "SOFTWARE\\Google\\Chrome\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host",
    ),
    (
        "Edge",
        "SOFTWARE\\Microsoft\\Edge\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host",
    ),
];

pub trait AppConfigProvider {
    fn get_app_config_dir(&self) -> String;
    fn get_resource_dir(&self) -> Result<PathBuf, io::Error>;
}
impl AppConfigProvider for AppHandle {
    fn get_app_config_dir(&self) -> String {
        DirsSavePathResolver::default().root_dir()
    }
    fn get_resource_dir(&self) -> Result<PathBuf, io::Error> {
        self.path().resource_dir().map_err(io::Error::other)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPackageInfo {
    pub version: String,
    pub package_path: String,
    pub manifest_info: ExtensionManifestInfo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifestInfo {
    pub name: String,
    pub version: String,
    pub extension_id: String,
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKeyInfo {
    pub browser: String,
    pub key_path: String,
    pub value: Option<String>,
    pub exists: bool,
}
#[derive(Debug, thiserror::Error)]
pub enum ExtensionInstallerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("{0}")]
    Package(String),
}

pub fn extension_id_from_key(key: &str) -> Result<String, ExtensionInstallerError> {
    let der = base64::engine::general_purpose::STANDARD
        .decode(key)
        .map_err(|e| {
            ExtensionInstallerError::Package(format!("拡張機能の公開鍵が不正です: {e}"))
        })?;
    if der.is_empty() {
        return Err(ExtensionInstallerError::Package(
            "拡張機能の公開鍵が空です".into(),
        ));
    }
    Ok(Sha256::digest(der)[..16]
        .iter()
        .flat_map(|byte| {
            [
                char::from(b'a' + (byte >> 4)),
                char::from(b'a' + (byte & 15)),
            ]
        })
        .collect())
}

#[derive(new)]
pub struct ExtensionInstallerUseCase<T: AppConfigProvider> {
    config_provider: Arc<T>,
}

impl<T: AppConfigProvider> ExtensionInstallerUseCase<T> {
    fn bundled_dir(&self) -> Result<PathBuf, ExtensionInstallerError> {
        Ok(self
            .config_provider
            .get_resource_dir()?
            .join("integrations"))
    }
    pub fn get_package_dir(&self) -> PathBuf {
        PathBuf::from(self.config_provider.get_app_config_dir()).join("browser-extension")
    }
    pub fn get_package_path(&self) -> PathBuf {
        self.get_package_dir().join("launcherg-extension.zip")
    }
    pub fn get_dev_extension_path(&self) -> PathBuf {
        // Stable path keeps the unpacked extension's browser registration valid on updates.
        self.get_package_dir().join("dev-extension")
    }
    pub fn is_dev_extension_available(&self) -> bool {
        self.get_dev_extension_path()
            .join("manifest.json")
            .is_file()
    }
    pub fn is_package_available(&self) -> bool {
        self.get_package_path().is_file()
    }
    pub fn get_package_size(&self) -> Result<u64, ExtensionInstallerError> {
        Ok(fs::metadata(self.get_package_path())?.len())
    }
    pub async fn get_extension_manifest_info(
        &self,
    ) -> Result<ExtensionManifestInfo, ExtensionInstallerError> {
        let path = self.bundled_dir()?.join("browser-extension/manifest.json");
        let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        let field = |name: &str| -> Result<String, ExtensionInstallerError> {
            manifest[name].as_str().map(str::to_owned).ok_or_else(|| {
                ExtensionInstallerError::Package(format!("拡張機能の{name}がありません"))
            })
        };
        Ok(ExtensionManifestInfo {
            name: field("name")?,
            version: field("version")?,
            extension_id: extension_id_from_key(&field("key")?)?,
            description: field("description")?,
        })
    }
    pub async fn generate_extension_package(
        &self,
    ) -> Result<ExtensionPackageInfo, ExtensionInstallerError> {
        let manifest_info = self.get_extension_manifest_info().await?;
        let source = self.bundled_dir()?.join("browser-extension");
        fs::create_dir_all(self.get_package_dir())?;
        let mut temp = tempfile::NamedTempFile::new_in(self.get_package_dir())?;
        {
            let mut archive = zip::ZipWriter::new(temp.as_file_mut());
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for entry in walkdir::WalkDir::new(&source).sort_by_file_name() {
                let entry = entry.map_err(|e| ExtensionInstallerError::Package(e.to_string()))?;
                if entry.file_type().is_file() {
                    let relative = entry
                        .path()
                        .strip_prefix(&source)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    archive.start_file(relative, options)?;
                    io::copy(&mut fs::File::open(entry.path())?, &mut archive)?;
                }
            }
            archive.finish()?;
        }
        temp.persist(self.get_package_path()).map_err(|e| e.error)?;
        Ok(ExtensionPackageInfo {
            version: manifest_info.version.clone(),
            package_path: self.get_package_path().to_string_lossy().into(),
            manifest_info,
        })
    }
    pub async fn copy_extension_for_development(&self) -> Result<String, ExtensionInstallerError> {
        self.get_extension_manifest_info().await?;
        let source = self.bundled_dir()?.join("browser-extension");
        let destination = self.get_dev_extension_path();
        copy_directory(&source, &destination)?;
        Ok(destination.to_string_lossy().into())
    }
    pub async fn setup_native_messaging_host(
        &self,
        extension_id: Option<String>,
    ) -> Result<String, ExtensionInstallerError> {
        let info = self.get_extension_manifest_info().await?;
        let extension_id = extension_id.unwrap_or(info.extension_id);
        if extension_id.len() != 32 || !extension_id.bytes().all(|b| (b'a'..=b'p').contains(&b)) {
            return Err(ExtensionInstallerError::Package(
                "拡張IDはa〜pの32文字で指定してください".into(),
            ));
        }
        let source = self.bundled_dir()?.join("native-messaging-host.exe");
        if !source.is_file() {
            return Err(ExtensionInstallerError::Package(
                "同梱の連携プログラムがありません。Launchergを再インストールしてください".into(),
            ));
        }
        // A versioned directory avoids overwriting a host while Chrome is running it.
        let version = info.version;
        if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err(ExtensionInstallerError::Package(
                "拡張機能のバージョンが不正です".into(),
            ));
        }
        let host_dir = self.get_package_dir().join("native-host").join(version);
        fs::create_dir_all(&host_dir)?;
        let executable = host_dir.join("native-messaging-host.exe");
        if !executable.is_file() || fs::read(&source)? != fs::read(&executable)? {
            fs::copy(&source, &executable)?;
        }
        let manifest_path = self
            .get_package_dir()
            .join("native-messaging-manifest.json");
        let manifest = serde_json::json!({
            "name": HOST_NAME, "description": "Launcherg browser integration",
            "path": executable, "type": "stdio",
            "allowed_origins": [format!("chrome-extension://{extension_id}/")]
        });
        // Native JSON and per-user registry writes avoid PowerShell and administrator access.
        fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        self.register_host(&manifest_path)?;
        Ok("Chrome / Edge の連携設定が完了しました".into())
    }
    #[cfg(windows)]
    fn register_host(&self, manifest: &Path) -> Result<(), ExtensionInstallerError> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        for (_, path) in REGISTRY_KEYS {
            user.create_subkey(path)?
                .0
                .set_value("", &manifest.to_string_lossy().as_ref())?;
        }
        Ok(())
    }
    #[cfg(not(windows))]
    fn register_host(&self, _: &Path) -> Result<(), ExtensionInstallerError> {
        Err(ExtensionInstallerError::Package("Windows専用です".into()))
    }
    #[cfg(windows)]
    pub fn check_registry_keys(&self) -> Result<Vec<RegistryKeyInfo>, ExtensionInstallerError> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        Ok(REGISTRY_KEYS
            .iter()
            .map(|(browser, path)| {
                let key = user.open_subkey(path);
                RegistryKeyInfo {
                    browser: browser.to_string(),
                    key_path: format!("HKCU\\{path}"),
                    exists: key.is_ok(),
                    value: key.ok().and_then(|key| key.get_value("").ok()),
                }
            })
            .collect())
    }
    #[cfg(windows)]
    pub fn remove_registry_keys(&self) -> Result<Vec<String>, ExtensionInstallerError> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let mut removed = Vec::new();
        for (browser, path) in REGISTRY_KEYS {
            match user.delete_subkey(path) {
                Ok(()) => removed.push(format!("{browser}の連携設定を解除しました")),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(removed)
    }
    #[cfg(not(windows))]
    pub fn check_registry_keys(&self) -> Result<Vec<RegistryKeyInfo>, ExtensionInstallerError> {
        Ok(vec![])
    }
    #[cfg(not(windows))]
    pub fn remove_registry_keys(&self) -> Result<Vec<String>, ExtensionInstallerError> {
        Ok(vec![])
    }
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), ExtensionInstallerError> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.map_err(|e| ExtensionInstallerError::Package(e.to_string()))?;
        let relative = entry.path().strip_prefix(source).unwrap();
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "extension_installer_test.rs"]
mod tests;
