use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::{fs, io};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::infrastructure::util::get_save_root_abs_dir_with_ptr_handle;

// AppHandleの依存を抽象化するtrait
pub trait AppConfigProvider {
    fn get_app_config_dir(&self) -> String;
}

// AppHandleの実装
impl AppConfigProvider for AppHandle {
    fn get_app_config_dir(&self) -> String {
        get_save_root_abs_dir_with_ptr_handle(self)
    }
}

const PROCTAIL_DIR: &str = "proctail";
const PROCTAIL_EXECUTABLE: &str = "ProcTail.Host.exe";
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/ryoha000/ProcTail/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcTailVersion {
    pub version: String,
    pub download_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcTailManagerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ProcTail process error: {0}")]
    Process(String),
    #[error("Download error: {0}")]
    Download(String),
}

pub struct ProcTailManager<T: AppConfigProvider> {
    config_provider: Arc<T>,
    process: Arc<Mutex<Option<Child>>>,
}

impl<T: AppConfigProvider> ProcTailManager<T> {
    pub fn new(config_provider: Arc<T>) -> Self {
        Self {
            config_provider,
            process: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_proctail_dir(&self) -> PathBuf {
        let root_dir = self.config_provider.get_app_config_dir();
        Path::new(&root_dir).join(PROCTAIL_DIR)
    }

    pub fn get_proctail_version_dir(&self, version: &str) -> PathBuf {
        let proctail_dir = self.get_proctail_dir();
        proctail_dir.join(format!("{}", version))
    }

    pub fn get_proctail_executable_path(&self, version: &str) -> PathBuf {
        self.get_proctail_version_dir(version).join("host").join(PROCTAIL_EXECUTABLE)
    }

    pub async fn get_current_proctail_executable_path(&self) -> Result<PathBuf, ProcTailManagerError> {
        let current_version = self.get_current_version().await?
            .ok_or_else(|| ProcTailManagerError::Process("No version installed".to_string()))?;
        Ok(self.get_proctail_executable_path(&current_version))
    }


    pub async fn get_current_version(&self) -> Result<Option<String>, ProcTailManagerError> {
        let proctail_dir = self.get_proctail_dir();
        if !proctail_dir.exists() {
            return Ok(None);
        }

        // Check for version directories
        let entries = fs::read_dir(proctail_dir)?;
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                // Return the first version directory found
                return Ok(Some(dir_name));
            }
        }
        
        Ok(None)
    }

    pub async fn get_latest_version(&self) -> Result<ProcTailVersion, ProcTailManagerError> {
        let client = reqwest::Client::new();
        let response = client
            .get(GITHUB_RELEASES_URL)
            .header("User-Agent", "launcherg")
            .send()
            .await?;

        let release_info: serde_json::Value = response.json().await?;
        
        let version = release_info["tag_name"]
            .as_str()
            .ok_or_else(|| ProcTailManagerError::Download("No version found".to_string()))?
            .to_string();

        let assets = release_info["assets"]
            .as_array()
            .ok_or_else(|| ProcTailManagerError::Download("No assets found".to_string()))?;

        let windows_asset = assets
            .iter()
            .find(|asset| {
                asset["name"]
                    .as_str()
                    .map_or(false, |name| name.contains("self-contained-win-x64") && name.ends_with(".zip"))
            })
            .ok_or_else(|| ProcTailManagerError::Download("No Windows asset found".to_string()))?;

        let download_url = windows_asset["browser_download_url"]
            .as_str()
            .ok_or_else(|| ProcTailManagerError::Download("No download URL found".to_string()))?
            .to_string();

        Ok(ProcTailVersion {
            version,
            download_url,
        })
    }

    pub async fn download_and_install(&self, version_info: &ProcTailVersion) -> Result<(), ProcTailManagerError> {
        let version_dir = self.get_proctail_version_dir(&version_info.version);
        fs::create_dir_all(&version_dir)?;

        // Download the zip file
        let client = reqwest::Client::new();
        let response = client
            .get(&version_info.download_url)
            .header("User-Agent", "launcherg")
            .send()
            .await?;

        let zip_data = response.bytes().await?;
        
        // Extract the zip file
        let cursor = std::io::Cursor::new(zip_data);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| ProcTailManagerError::Download(format!("Failed to open zip: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| ProcTailManagerError::Download(format!("Failed to read zip entry: {}", e)))?;
            
            let file_name = file.name().to_string();
            let file_path = version_dir.join(&file_name);

            if file.is_dir() {
                fs::create_dir_all(&file_path)?;
            } else {
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&file_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        // No need to write version file as version is stored in directory name

        Ok(())
    }

    pub async fn is_update_available(&self) -> Result<bool, ProcTailManagerError> {
        let current_version = self.get_current_version().await?;
        let latest_version = self.get_latest_version().await?;

        match current_version {
            Some(current) => Ok(current != latest_version.version),
            None => Ok(true), // No version installed, update available
        }
    }

    pub async fn ensure_latest_version(&self) -> Result<(), ProcTailManagerError> {
        if self.is_update_available().await? {
            let latest_version = self.get_latest_version().await?;
            self.download_and_install(&latest_version).await?;
        }
        Ok(())
    }

    pub async fn start_proctail(&self) -> Result<(), ProcTailManagerError> {
        // Ensure we have the latest version
        self.ensure_latest_version().await?;

        let executable_path = self.get_current_proctail_executable_path().await?;
        if !executable_path.exists() {
            return Err(ProcTailManagerError::Process(
                "ProcTail executable not found".to_string(),
            ));
        }

        let mut process_guard = self.process.lock().await;
        
        // Check if process is already running
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited, remove it
                    *process_guard = None;
                }
                Ok(None) => {
                    // Process is still running
                    return Ok(());
                }
                Err(e) => {
                    return Err(ProcTailManagerError::Process(format!("Failed to check process status: {}", e)));
                }
            }
        }

        // Start new process
        let child = Command::new(executable_path)
            .spawn()
            .map_err(|e| ProcTailManagerError::Process(format!("Failed to start ProcTail: {}", e)))?;

        *process_guard = Some(child);
        Ok(())
    }

    pub async fn stop_proctail(&self) -> Result<(), ProcTailManagerError> {
        let mut process_guard = self.process.lock().await;
        
        if let Some(mut child) = process_guard.take() {
            child.kill()
                .map_err(|e| ProcTailManagerError::Process(format!("Failed to kill ProcTail: {}", e)))?;
            
            child.wait()
                .map_err(|e| ProcTailManagerError::Process(format!("Failed to wait for ProcTail: {}", e)))?;
        }
        
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().await;
        
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    *process_guard = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking process, assume not running
                    *process_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub async fn get_status(&self) -> Result<ProcTailManagerStatus, ProcTailManagerError> {
        let current_version = self.get_current_version().await?;
        let is_running = self.is_running().await;
        let executable_exists = if let Ok(path) = self.get_current_proctail_executable_path().await {
            path.exists()
        } else {
            false
        };
        
        let update_available = if executable_exists {
            self.is_update_available().await.unwrap_or(false)
        } else {
            true
        };

        Ok(ProcTailManagerStatus {
            current_version,
            is_running,
            executable_exists,
            update_available,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcTailManagerStatus {
    pub current_version: Option<String>,
    pub is_running: bool,
    pub executable_exists: bool,
    pub update_available: bool,
}

// AppHandleを使う場合の型エイリアス
pub type AppHandleProcTailManager = ProcTailManager<AppHandle>;

