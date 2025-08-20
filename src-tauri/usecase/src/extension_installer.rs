use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};
use serde::{Deserialize, Serialize};
use derive_new::new;
use std::sync::Arc;
use tauri::AppHandle;
use std::io::Write;
use tempfile::NamedTempFile;

use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// AppHandleの依存を抽象化するtrait
pub trait AppConfigProvider {
    fn get_app_config_dir(&self) -> String;
}

// AppHandleの実装
impl AppConfigProvider for AppHandle {
    fn get_app_config_dir(&self) -> String {
        DirsSavePathResolver::default().root_dir()
    }
}

const BROWSER_EXTENSION_DIR: &str = "browser-extension";
const EXTENSION_PACKAGE_NAME: &str = "launcherg-extension.zip";
const CHROME_REGISTRY_KEY: &str = "SOFTWARE\\Google\\Chrome\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host";
const EDGE_REGISTRY_KEY: &str = "SOFTWARE\\Microsoft\\Edge\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host";

// PowerShellスクリプトを文字列として埋め込み
const INSTALL_SCRIPT: &str = r#"# Native Messaging Host インストールスクリプト (Windows)
# 管理者権限で実行する必要があります

param(
    [Parameter(Mandatory=$true)]
    [string]$ExtensionId,
    
    [Parameter(Mandatory=$false)]
    [string]$InstallPath = $PSScriptRoot
)

$ErrorActionPreference = "Stop"

# マニフェストファイルのパス
$manifestPath = Join-Path $InstallPath "native-messaging-manifest.json"
$exePath = Join-Path $InstallPath "native-messaging-host.exe"

# マニフェストファイルの存在確認
if (-not (Test-Path $manifestPath)) {
    Write-Error "Manifest file not found: $manifestPath"
    exit 1
}

# 実行ファイルの存在確認
if (-not (Test-Path $exePath)) {
    Write-Error "Executable file not found: $exePath"
    exit 1
}

# マニフェストを読み込み
$manifest = Get-Content $manifestPath | ConvertFrom-Json

# Extension IDを更新
$manifest.allowed_origins = @("chrome-extension://$ExtensionId/")
$manifest.path = $exePath

# 更新されたマニフェストを保存
$updatedManifestPath = Join-Path $InstallPath "native-messaging-manifest-installed.json"
$manifest | ConvertTo-Json -Depth 10 | Set-Content $updatedManifestPath -Encoding UTF8

Write-Host "Manifest file created: $updatedManifestPath"

# レジストリキーのパス
$chromeRegPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\moe.ryoha.launcherg.extension_host"
$edgeRegPath = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\moe.ryoha.launcherg.extension_host"

# Chrome用レジストリエントリ作成
try {
    New-Item -Path $chromeRegPath -Force | Out-Null
    Set-ItemProperty -Path $chromeRegPath -Name "(Default)" -Value $updatedManifestPath
    Write-Host "Chrome registry entry created successfully"
} catch {
    Write-Warning "Failed to create Chrome registry entry: $_"
}

# Edge用レジストリエントリ作成
try {
    New-Item -Path $edgeRegPath -Force | Out-Null
    Set-ItemProperty -Path $edgeRegPath -Name "(Default)" -Value $updatedManifestPath
    Write-Host "Edge registry entry created successfully"
} catch {
    Write-Warning "Failed to create Edge registry entry: $_"
}

Write-Host ""
Write-Host "Native Messaging Host installation completed!"
Write-Host ""
Write-Host "Extension ID: $ExtensionId"
Write-Host "Manifest path: $updatedManifestPath"
Write-Host "Executable path: $exePath"
Write-Host ""
Write-Host "To uninstall, run: .\uninstall-native-messaging-host.ps1"
"#;

const AUTO_INSTALL_SCRIPT: &str = r#"# Native Messaging Host Auto-Install Script
# Automatically detects Extension ID and performs installation

param(
    [Parameter(Mandatory=$false)]
    [string]$InstallPath = $PSScriptRoot
)

$ErrorActionPreference = "Stop"

Write-Host "Launcherg Native Messaging Host Auto-Install"
Write-Host "============================================"
Write-Host ""

# Function to auto-detect Chrome extension ID
function Find-ExtensionId {
    param(
        [string]$BrowserName,
        [string]$ExtensionsPath
    )
    
    if (-not (Test-Path $ExtensionsPath)) {
        Write-Host "  $BrowserName extensions folder not found: $ExtensionsPath"
        return $null
    }
    
    Write-Host "  Searching $BrowserName extensions..."
    
    # Search extension folders
    $extensionDirs = Get-ChildItem -Path $ExtensionsPath -Directory | Where-Object {
        $_.Name -match '^[a-p]{32}$'  # Chrome extension ID format
    }
    
    foreach ($dir in $extensionDirs) {
        $manifestPath = Join-Path $dir.FullName "*\manifest.json"
        $manifestFiles = Get-ChildItem -Path $manifestPath -ErrorAction SilentlyContinue
        
        foreach ($manifestFile in $manifestFiles) {
            try {
                $manifest = Get-Content $manifestFile.FullName | ConvertFrom-Json
                if ($manifest.name -eq "Launcherg DL Store Sync") {
                    Write-Host "    Found: $($dir.Name)"
                    return $dir.Name
                }
            }
            catch {
                # Skip if manifest.json cannot be read
                continue
            }
        }
    }
    
    return $null
}

# Chrome/Edge extension search paths
$chromeExtensionsPath = "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Extensions"
$edgeExtensionsPath = "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Default\Extensions"

Write-Host "1. Searching for extension ID..."

# Search Chrome extensions
$chromeExtensionId = Find-ExtensionId -BrowserName "Chrome" -ExtensionsPath $chromeExtensionsPath

# Search Edge extensions
$edgeExtensionId = Find-ExtensionId -BrowserName "Edge" -ExtensionsPath $edgeExtensionsPath

# Display found extension IDs
if ($chromeExtensionId) {
    Write-Host "  Chrome Extension ID: $chromeExtensionId"
}
if ($edgeExtensionId) {
    Write-Host "  Edge Extension ID: $edgeExtensionId"
}

# Check if at least one ID was found
$foundExtensionId = $null
if ($chromeExtensionId) {
    $foundExtensionId = $chromeExtensionId
} elseif ($edgeExtensionId) {
    $foundExtensionId = $edgeExtensionId
}

if (-not $foundExtensionId) {
    Write-Host ""
    Write-Host "Error: Launcherg DL Store Sync extension was not found."
    Write-Host "Please install the browser extension first, then run this script again."
    Write-Host ""
    Write-Host "Installation cancelled."
    exit 1
}

Write-Host ""
Write-Host "2. Installing Native Messaging Host..."

# Execute main installation script inline
try {
    # 以下はinstall-native-messaging-host.ps1の内容をインラインで実行
    $ExtensionId = $foundExtensionId
    
    # マニフェストファイルのパス
    $manifestPath = Join-Path $InstallPath "native-messaging-manifest.json"
    $exePath = Join-Path $InstallPath "native-messaging-host.exe"
    
    # マニフェストファイルの存在確認
    if (-not (Test-Path $manifestPath)) {
        Write-Error "Manifest file not found: $manifestPath"
        exit 1
    }
    
    # 実行ファイルの存在確認
    if (-not (Test-Path $exePath)) {
        Write-Error "Executable file not found: $exePath"
        exit 1
    }
    
    # マニフェストを読み込み
    $manifest = Get-Content $manifestPath | ConvertFrom-Json
    
    # Extension IDを更新
    $manifest.allowed_origins = @("chrome-extension://$ExtensionId/")
    $manifest.path = $exePath
    
    # 更新されたマニフェストを保存
    $updatedManifestPath = Join-Path $InstallPath "native-messaging-manifest-installed.json"
    $manifest | ConvertTo-Json -Depth 10 | Set-Content $updatedManifestPath -Encoding UTF8
    
    Write-Host "Manifest file created: $updatedManifestPath"
    
    # レジストリキーのパス
    $chromeRegPath = "HKCU:\Software\Google\Chrome\NativeMessagingHosts\moe.ryoha.launcherg.extension_host"
    $edgeRegPath = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\moe.ryoha.launcherg.extension_host"
    
    # Chrome用レジストリエントリ作成
    try {
        New-Item -Path $chromeRegPath -Force | Out-Null
        Set-ItemProperty -Path $chromeRegPath -Name "(Default)" -Value $updatedManifestPath
        Write-Host "Chrome registry entry created successfully"
    } catch {
        Write-Warning "Failed to create Chrome registry entry: $_"
    }
    
    # Edge用レジストリエントリ作成
    try {
        New-Item -Path $edgeRegPath -Force | Out-Null
        Set-ItemProperty -Path $edgeRegPath -Name "(Default)" -Value $updatedManifestPath
        Write-Host "Edge registry entry created successfully"
    } catch {
        Write-Warning "Failed to create Edge registry entry: $_"
    }
    
    Write-Host ""
    Write-Host "Native Messaging Host installation completed!"
    Write-Host ""
    Write-Host "Extension ID: $ExtensionId"
    Write-Host "Manifest path: $updatedManifestPath"
    Write-Host "Executable path: $exePath"
    
    Write-Host ""
    Write-Host "Installation completed successfully!"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "1. Start the Launcherg desktop application"
    Write-Host "2. Open DMM Games or DLsite library page"
    Write-Host "3. Click the extension icon and test 'Manual Sync'"
    Write-Host ""
}
catch {
    Write-Host "Installation error occurred: $($_.Exception.Message)"
    exit 1
}
"#;

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
    #[error("Build error: {0}")]
    Build(String),
    #[error("Package error: {0}")]
    Package(String),
    #[error("PowerShell error: {0}")]
    PowerShell(String),
}

#[derive(new)]
pub struct ExtensionInstallerUseCase<T: AppConfigProvider> {
    config_provider: Arc<T>,
}

impl<T: AppConfigProvider> ExtensionInstallerUseCase<T> {
    /// 拡張機能のソースディレクトリを取得
    pub fn get_extension_source_dir(&self) -> PathBuf {
        // このリポジトリのbrowser-extensionディレクトリ
        PathBuf::from(r"F:\workspace\launcherg\browser-extension")
    }

    /// 拡張機能パッケージの保存ディレクトリを取得
    pub fn get_package_dir(&self) -> PathBuf {
        let root_dir = self.config_provider.get_app_config_dir();
        Path::new(&root_dir).join(BROWSER_EXTENSION_DIR)
    }

    /// 拡張機能パッケージのパスを取得
    pub fn get_package_path(&self) -> PathBuf {
        self.get_package_dir().join(EXTENSION_PACKAGE_NAME)
    }

    /// 拡張機能のmanifest.jsonを読み込んでメタデータを取得
    pub async fn get_extension_manifest_info(&self) -> Result<ExtensionManifestInfo, ExtensionInstallerError> {
        let source_dir = self.get_extension_source_dir();
        let manifest_path = source_dir.join("manifest.json");

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

        // Extension IDを計算（manifest.jsonのkeyフィールドから）
        let extension_id = if let Some(key) = manifest.get("key").and_then(|k| k.as_str()) {
            self.calculate_extension_id_from_key(key)?
        } else {
            return Err(ExtensionInstallerError::Package(
                "Extension key is required in manifest.json".to_string()
            ));
        };

        Ok(ExtensionManifestInfo {
            name: manifest.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Launcherg DL Store Sync")
                .to_string(),
            version: manifest.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0")
                .to_string(),
            extension_id,
            description: manifest.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    /// 拡張機能をビルドしてZIPパッケージを生成
    pub async fn generate_extension_package(&self) -> Result<ExtensionPackageInfo, ExtensionInstallerError> {
        let source_dir = self.get_extension_source_dir();
        let package_dir = self.get_package_dir();
        let package_path = self.get_package_path();

        // パッケージディレクトリを作成
        fs::create_dir_all(&package_dir)?;

        // 既存のパッケージがあれば削除
        if package_path.exists() {
            fs::remove_file(&package_path)?;
        }

        // Windows環境でのnpmコマンド名を決定
        let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
        
        // npmの利用可能性をチェック
        let npm_check = Command::new(npm_cmd)
            .args(&["--version"])
            .output();
            
        match npm_check {
            Ok(output) if output.status.success() => {
                // npm利用可能
            },
            _ => {
                return Err(ExtensionInstallerError::Build(
                    "npm が見つかりません。Node.js がインストールされていることを確認してください。".to_string()
                ));
            }
        }

        // npm install
        let npm_install = Command::new(npm_cmd)
            .args(&["install"])
            .current_dir(&source_dir)
            .output()?;

        if !npm_install.status.success() {
            let error_msg = String::from_utf8_lossy(&npm_install.stderr);
            return Err(ExtensionInstallerError::Build(format!("npm install failed: {}", error_msg)));
        }

        // npm run build
        let npm_build = Command::new(npm_cmd)
            .args(&["run", "build"])
            .current_dir(&source_dir)
            .output()?;

        if !npm_build.status.success() {
            let error_msg = String::from_utf8_lossy(&npm_build.stderr);
            return Err(ExtensionInstallerError::Build(format!("npm run build failed: {}", error_msg)));
        }

        // distディレクトリをZIPに圧縮
        let dist_dir = source_dir.join("dist");
        if !dist_dir.exists() {
            return Err(ExtensionInstallerError::Package("dist directory not found after build".to_string()));
        }

        self.create_zip_from_directory(&dist_dir, &package_path)?;

        // マニフェスト情報を取得
        let manifest_info = self.get_extension_manifest_info().await?;

        Ok(ExtensionPackageInfo {
            version: manifest_info.version.clone(),
            package_path: package_path.to_string_lossy().to_string(),
            manifest_info,
        })
    }

    /// ディレクトリをZIPファイルに圧縮
    fn create_zip_from_directory(&self, source_dir: &Path, output_path: &Path) -> Result<(), ExtensionInstallerError> {
        let file = fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        self.add_directory_to_zip(&mut zip, source_dir, "", options)?;
        zip.finish()?;

        Ok(())
    }

    /// ディレクトリの内容を再帰的にZIPに追加
    fn add_directory_to_zip<W: std::io::Write + std::io::Seek>(
        &self,
        zip: &mut zip::ZipWriter<W>,
        dir_path: &Path,
        prefix: &str,
        options: zip::write::FileOptions,
    ) -> Result<(), ExtensionInstallerError> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if path.is_file() {
                let file_path = if prefix.is_empty() {
                    name_str.to_string()
                } else {
                    format!("{}/{}", prefix, name_str)
                };

                zip.start_file(file_path, options)?;
                let mut file = fs::File::open(&path)?;
                std::io::copy(&mut file, zip)?;
            } else if path.is_dir() {
                let dir_path_in_zip = if prefix.is_empty() {
                    name_str.to_string()
                } else {
                    format!("{}/{}", prefix, name_str)
                };
                self.add_directory_to_zip(zip, &path, &dir_path_in_zip, options)?;
            }
        }

        Ok(())
    }

    /// 公開鍵からExtension IDを計算
    fn calculate_extension_id_from_key(&self, _key: &str) -> Result<String, ExtensionInstallerError> {
        // 実際の計算は複雑なため、一旦エラーを返す
        // 本来はSHA256ハッシュの計算が必要
        return Err(ExtensionInstallerError::Package(
            "Extension ID calculation not implemented. Please specify extension ID manually.".to_string()
        ));
    }

    /// PowerShellスクリプトを実行してNative Messaging Hostをセットアップ
    pub async fn setup_native_messaging_host(&self, extension_id: Option<String>) -> Result<String, ExtensionInstallerError> {
        let source_dir = self.get_extension_source_dir();
        let parent_dir = source_dir.parent()
            .ok_or_else(|| ExtensionInstallerError::PowerShell("Cannot find parent directory".to_string()))?;
        let tauri_dir = parent_dir.join("src-tauri");

        // Native Messaging Hostの実行ファイルが存在するかチェック
        let exe_path = tauri_dir.join("native-messaging-host.exe");
        let target_exe_path = tauri_dir.join("target").join("release").join("native-messaging-host.exe");
        let debug_exe_path = tauri_dir.join("target").join("debug").join("native-messaging-host.exe");

        if !exe_path.exists() && !target_exe_path.exists() && !debug_exe_path.exists() {
            println!("Native Messaging Host executable not found, building...");
            
            // Cargoでnative-messaging-hostをビルド
            let cargo_build = Command::new("cargo")
                .args(&["build", "--release", "--bin", "native-messaging-host"])
                .current_dir(&tauri_dir)
                .output()?;

            if !cargo_build.status.success() {
                let error_msg = String::from_utf8_lossy(&cargo_build.stderr);
                return Err(ExtensionInstallerError::Build(
                    format!("Failed to build native-messaging-host: {}", error_msg)
                ));
            }

            // ビルドされた実行ファイルをtauriディレクトリにコピー
            if target_exe_path.exists() {
                fs::copy(&target_exe_path, &exe_path)?;
                println!("Copied native-messaging-host.exe to tauri directory");
            } else {
                return Err(ExtensionInstallerError::Build(
                    "Built executable not found in target/release directory".to_string()
                ));
            }
        }

        // PowerShellスクリプトを実行（詳細なログ付き）
        let script_content = if let Some(ext_id) = extension_id {
            // Extension IDが指定されている場合は、直接インストールスクリプトを実行
            println!("Executing PowerShell script with Extension ID: {}", ext_id);
            // スクリプト内の$PSScriptRootを実際のパスに置換
            INSTALL_SCRIPT.replace("$PSScriptRoot", tauri_dir.to_str().unwrap())
                .replace("[Parameter(Mandatory=$true)]", "[Parameter(Mandatory=$false)]")
                + &format!("\n$ExtensionId = '{}'", ext_id)
        } else {
            // Extension IDが指定されていない場合は、auto-installスクリプトを使用
            println!("Executing auto-install PowerShell script");
            // スクリプト内の$PSScriptRootを実際のパスに置換
            AUTO_INSTALL_SCRIPT.replace("$PSScriptRoot", tauri_dir.to_str().unwrap())
        };
        
        // 一時ファイルにスクリプトを書き込み
        let mut temp_script = NamedTempFile::new()?;
        temp_script.write_all(script_content.as_bytes())?;
        temp_script.flush()?;
        let temp_script_path = temp_script.path();
        
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy", "Bypass",
                "-File",
                temp_script_path.to_str().unwrap()
            ])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        println!("PowerShell output:");
        println!("Exit code: {}", output.status.code().unwrap_or(-1));
        println!("Stdout: {}", stdout);
        println!("Stderr: {}", stderr);

        if !output.status.success() {
            return Err(ExtensionInstallerError::PowerShell(
                format!("PowerShell script failed:\nExit code: {}\nStdout: {}\nStderr: {}", 
                    output.status.code().unwrap_or(-1), stdout, stderr)
            ));
        }

        Ok(format!("セットアップが完了しました:\n{}", stdout))
    }

    /// パッケージが存在するかチェック
    pub fn is_package_available(&self) -> bool {
        self.get_package_path().exists()
    }

    /// パッケージファイルサイズを取得
    pub fn get_package_size(&self) -> Result<u64, ExtensionInstallerError> {
        let package_path = self.get_package_path();
        if !package_path.exists() {
            return Err(ExtensionInstallerError::Package("Package file not found".to_string()));
        }

        let metadata = fs::metadata(&package_path)?;
        Ok(metadata.len())
    }

    /// 開発環境用: browser-extension/dist を直接コピーしてローカル配布用フォルダを作成
    pub async fn copy_extension_for_development(&self) -> Result<String, ExtensionInstallerError> {
        let source_dir = self.get_extension_source_dir();
        let package_dir = self.get_package_dir();
        
        // 配布用ディレクトリを作成
        let dev_extension_dir = package_dir.join("dev-extension");
        
        // 既存のディレクトリがあれば削除
        if dev_extension_dir.exists() {
            fs::remove_dir_all(&dev_extension_dir)?;
        }
        fs::create_dir_all(&dev_extension_dir)?;

        // Windows環境でのnpmコマンド名を決定
        let npm_cmd = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };
        
        // npmの利用可能性をチェック
        let npm_check = Command::new(npm_cmd)
            .args(&["--version"])
            .output();
            
        match npm_check {
            Ok(output) if output.status.success() => {
                // npm利用可能
            },
            _ => {
                return Err(ExtensionInstallerError::Build(
                    "npm が見つかりません。Node.js がインストールされていることを確認してください。".to_string()
                ));
            }
        }

        // npm build を実行
        let npm_install = Command::new(npm_cmd)
            .args(&["install"])
            .current_dir(&source_dir)
            .output()?;

        if !npm_install.status.success() {
            let error_msg = String::from_utf8_lossy(&npm_install.stderr);
            return Err(ExtensionInstallerError::Build(format!("npm install failed: {}", error_msg)));
        }

        let npm_build = Command::new(npm_cmd)
            .args(&["run", "build"])
            .current_dir(&source_dir)
            .output()?;

        if !npm_build.status.success() {
            let error_msg = String::from_utf8_lossy(&npm_build.stderr);
            return Err(ExtensionInstallerError::Build(format!("npm run build failed: {}", error_msg)));
        }

        // distディレクトリの存在確認
        let dist_dir = source_dir.join("dist");
        if !dist_dir.exists() {
            return Err(ExtensionInstallerError::Package("dist directory not found after build".to_string()));
        }

        // distディレクトリの内容を再帰的にコピー
        self.copy_directory_contents(&dist_dir, &dev_extension_dir)?;

        Ok(dev_extension_dir.to_string_lossy().to_string())
    }

    /// ディレクトリの内容を再帰的にコピー
    fn copy_directory_contents(&self, src: &Path, dst: &Path) -> Result<(), ExtensionInstallerError> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(file_name);

            if src_path.is_file() {
                fs::copy(&src_path, &dst_path)?;
            } else if src_path.is_dir() {
                fs::create_dir_all(&dst_path)?;
                self.copy_directory_contents(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// 開発用拡張機能ディレクトリのパスを取得
    pub fn get_dev_extension_path(&self) -> PathBuf {
        self.get_package_dir().join("dev-extension")
    }

    /// 開発用拡張機能が存在するかチェック
    pub fn is_dev_extension_available(&self) -> bool {
        let dev_path = self.get_dev_extension_path();
        dev_path.exists() && dev_path.join("manifest.json").exists()
    }
    
    /// レジストリキーの状態を確認
    #[cfg(target_os = "windows")]
    pub fn check_registry_keys(&self) -> Result<Vec<RegistryKeyInfo>, ExtensionInstallerError> {
        let chrome_key = CHROME_REGISTRY_KEY;
        let edge_key = EDGE_REGISTRY_KEY;
        
        let mut results = Vec::new();
        
        // Chrome レジストリキーを確認
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let chrome_info = match hkcu.open_subkey(chrome_key) {
            Ok(subkey) => {
                let value: Result<String, _> = subkey.get_value("");
                RegistryKeyInfo {
                    browser: "Chrome".to_string(),
                    key_path: format!("HKCU\\{}", chrome_key),
                    value: value.ok(),
                    exists: true,
                }
            },
            Err(_) => RegistryKeyInfo {
                browser: "Chrome".to_string(),
                key_path: format!("HKCU\\{}", chrome_key),
                value: None,
                exists: false,
            }
        };
        results.push(chrome_info);
        
        // Edge レジストリキーを確認
        let edge_info = match hkcu.open_subkey(edge_key) {
            Ok(subkey) => {
                let value: Result<String, _> = subkey.get_value("");
                RegistryKeyInfo {
                    browser: "Edge".to_string(),
                    key_path: format!("HKCU\\{}", edge_key),
                    value: value.ok(),
                    exists: true,
                }
            },
            Err(_) => RegistryKeyInfo {
                browser: "Edge".to_string(),
                key_path: format!("HKCU\\{}", edge_key),
                value: None,
                exists: false,
            }
        };
        results.push(edge_info);
        
        Ok(results)
    }
    
    /// レジストリキーを削除
    #[cfg(target_os = "windows")]
    pub fn remove_registry_keys(&self) -> Result<Vec<String>, ExtensionInstallerError> {
        let chrome_key = CHROME_REGISTRY_KEY;
        let edge_key = EDGE_REGISTRY_KEY;
        
        let mut results = Vec::new();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        
        // Chrome レジストリキーを削除
        match hkcu.delete_subkey(chrome_key) {
            Ok(_) => results.push("Chrome registry key removed successfully".to_string()),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    results.push("Chrome registry key not found".to_string());
                } else {
                    results.push(format!("Failed to remove Chrome registry key: {}", e));
                }
            }
        }
        
        // Edge レジストリキーを削除
        match hkcu.delete_subkey(edge_key) {
            Ok(_) => results.push("Edge registry key removed successfully".to_string()),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    results.push("Edge registry key not found".to_string());
                } else {
                    results.push(format!("Failed to remove Edge registry key: {}", e));
                }
            }
        }
        
        // インストール済みマニフェストファイルも削除
        let source_dir = self.get_extension_source_dir();
        let parent_dir = source_dir.parent()
            .ok_or_else(|| ExtensionInstallerError::PowerShell("Cannot find parent directory".to_string()))?;
        let tauri_dir = parent_dir.join("src-tauri");
        let installed_manifest_path = tauri_dir.join("native-messaging-manifest-installed.json");
        
        if installed_manifest_path.exists() {
            match fs::remove_file(&installed_manifest_path) {
                Ok(_) => results.push("Installed manifest file removed".to_string()),
                Err(e) => results.push(format!("Failed to remove manifest file: {}", e)),
            }
        } else {
            results.push("Installed manifest file not found".to_string());
        }
        
        Ok(results)
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn check_registry_keys(&self) -> Result<Vec<RegistryKeyInfo>, ExtensionInstallerError> {
        Err(ExtensionInstallerError::PowerShell("Registry operations are only supported on Windows".to_string()))
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn remove_registry_keys(&self) -> Result<Vec<String>, ExtensionInstallerError> {
        Err(ExtensionInstallerError::PowerShell("Registry operations are only supported on Windows".to_string()))
    }
}