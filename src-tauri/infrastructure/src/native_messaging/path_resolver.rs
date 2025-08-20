use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Native Messaging Hostの実行ファイルパスを解決するユーティリティ
pub struct NativeHostPathResolver;

impl NativeHostPathResolver {
    /// レジストリからパスを解決する
    pub fn resolve_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        Self::get_path_from_registry()
    }

    /// レジストリからNative Messaging Hostのパスを取得
    #[cfg(target_os = "windows")]
    fn get_path_from_registry() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let chrome_key = "SOFTWARE\\Google\\Chrome\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host";
        let edge_key = "SOFTWARE\\Microsoft\\Edge\\NativeMessagingHosts\\moe.ryoha.launcherg.extension_host";
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut error_details = Vec::new();
        
        // Chromeレジストリを確認
        match hkcu.open_subkey(chrome_key) {
            Ok(subkey) => {
                log::debug!("Chrome registry key found: {}", chrome_key);
                match subkey.get_value::<String, _>("") {
                    Ok(manifest_path) => {
                        log::debug!("Chrome manifest path found: {}", manifest_path);
                        match Self::get_executable_path_from_manifest(&manifest_path) {
                            Ok(exe_path) => {
                                log::debug!("Chrome executable path resolved: {}", exe_path.display());
                                return Ok(exe_path);
                            }
                            Err(e) => {
                                let error = format!("Chrome manifest processing failed: {}", e);
                                log::warn!("{}", error);
                                error_details.push(error);
                            }
                        }
                    }
                    Err(e) => {
                        let error = format!("Chrome registry value read failed: {}", e);
                        log::warn!("{}", error);
                        error_details.push(error);
                    }
                }
            }
            Err(e) => {
                let error = format!("Chrome registry key not found ({}): {}", chrome_key, e);
                log::debug!("{}", error);
                error_details.push(error);
            }
        }
        
        // Edgeレジストリを確認
        match hkcu.open_subkey(edge_key) {
            Ok(subkey) => {
                log::debug!("Edge registry key found: {}", edge_key);
                match subkey.get_value::<String, _>("") {
                    Ok(manifest_path) => {
                        log::debug!("Edge manifest path found: {}", manifest_path);
                        match Self::get_executable_path_from_manifest(&manifest_path) {
                            Ok(exe_path) => {
                                log::debug!("Edge executable path resolved: {}", exe_path.display());
                                return Ok(exe_path);
                            }
                            Err(e) => {
                                let error = format!("Edge manifest processing failed: {}", e);
                                log::warn!("{}", error);
                                error_details.push(error);
                            }
                        }
                    }
                    Err(e) => {
                        let error = format!("Edge registry value read failed: {}", e);
                        log::warn!("{}", error);
                        error_details.push(error);
                    }
                }
            }
            Err(e) => {
                let error = format!("Edge registry key not found ({}): {}", edge_key, e);
                log::debug!("{}", error);
                error_details.push(error);
            }
        }

        let combined_error = format!(
            "No valid registry entry found. Detailed errors:\n{}",
            error_details.join("\n")
        );
        log::error!("{}", combined_error);
        Err(combined_error.into())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_path_from_registry() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        Err("Registry operations are only supported on Windows".into())
    }

    /// マニフェストファイルから実行ファイルのパスを読み取り
    fn get_executable_path_from_manifest(manifest_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Reading manifest file: {}", manifest_path);
        
        // ファイルの存在確認
        if !std::path::Path::new(manifest_path).exists() {
            let error = format!("Manifest file does not exist: {}", manifest_path);
            log::error!("{}", error);
            return Err(error.into());
        }

        // ファイル読み取り
        let mut manifest_content = std::fs::read_to_string(manifest_path)
            .map_err(|e| {
                let error = format!("Failed to read manifest file {}: {} (kind: {:?})", manifest_path, e, e.kind());
                log::error!("{}", error);
                error
            })?;

        log::debug!("Manifest content length: {} bytes", manifest_content.len());

        // BOM（Byte Order Mark）を除去
        if manifest_content.starts_with('\u{FEFF}') {
            log::debug!("Removing BOM from manifest file");
            manifest_content = manifest_content.trim_start_matches('\u{FEFF}').to_string();
        }

        // JSON解析
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| {
                let error = format!("Failed to parse manifest JSON from {}: {}\nContent preview: {}", 
                    manifest_path, e, 
                    if manifest_content.len() > 200 { 
                        format!("{}...", &manifest_content[..200]) 
                    } else { 
                        manifest_content.clone() 
                    }
                );
                log::error!("{}", error);
                error
            })?;

        log::debug!("Parsed manifest structure: {}", serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "Failed to serialize".to_string()));

        // pathフィールドの取得
        let exe_path_str = manifest.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| {
                let error = format!("No 'path' field in manifest {}. Available fields: {:?}", 
                    manifest_path, 
                    manifest.as_object().map(|obj| obj.keys().collect::<Vec<_>>()).unwrap_or_default()
                );
                log::error!("{}", error);
                error
            })?;

        log::debug!("Executable path from manifest: {}", exe_path_str);

        let exe_path = PathBuf::from(exe_path_str);
        
        // 実行ファイルの存在確認
        if !exe_path.exists() {
            let error = format!("Executable not found: {} (absolute path: {})", 
                exe_path.display(), 
                exe_path.canonicalize().unwrap_or_else(|_| exe_path.clone()).display()
            );
            log::error!("{}", error);
            return Err(error.into());
        }

        log::debug!("Executable found and verified: {}", exe_path.display());
        Ok(exe_path)
    }

}