use std::fs;

use crate::domain::extension::{ExtensionConfig as DomainExtensionConfig, ExtensionConnectionStatus};

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct HostStatusStore {
    last_sync_seconds: Option<i64>,
    total_synced: u32,
    recent_extension_ids: Vec<String>,
}

fn host_root_dir() -> String {
    // %APPDATA%\ryoha.moe\launcherg
    let base = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap());
    let path = base.join("ryoha.moe").join("launcherg");
    fs::create_dir_all(&path).ok();
    path.to_string_lossy().to_string()
}

fn status_file_path() -> String { format!("{}/native_host_status.json", host_root_dir()) }
fn config_file_path() -> String { format!("{}/native_host_config.json", host_root_dir()) }
pub fn db_file_path() -> String { format!("{}/launcherg_sqlite.db3", host_root_dir()) }

fn load_status_store() -> HostStatusStore {
    let p = status_file_path();
    match fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => HostStatusStore::default(),
    }
}

fn save_status_store(store: HostStatusStore) {
    let p = status_file_path();
    let _ = fs::write(p, serde_json::to_string_pretty(&store).unwrap_or("{}".to_string()));
}

pub fn save_config(config: &DomainExtensionConfig) -> anyhow::Result<()> {
    let p = config_file_path();
    fs::write(p, serde_json::to_string_pretty(config).unwrap_or("{}".to_string()))?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct HostStatusData {
    pub last_sync_seconds: Option<i64>,
    pub total_synced: u32,
    pub connected_extensions: Vec<String>,
}

pub fn get_status_data() -> HostStatusData {
    let s = load_status_store();
    HostStatusData {
        last_sync_seconds: s.last_sync_seconds,
        total_synced: s.total_synced,
        connected_extensions: s.recent_extension_ids,
    }
}

pub fn bump_sync_counters(success_add: u32) {
    let mut s = load_status_store();
    s.last_sync_seconds = Some(chrono::Utc::now().timestamp());
    s.total_synced = s.total_synced.saturating_add(success_add);
    save_status_store(s);
}


