use domain::windows::shell_link::ShellLink as _;
use domain::windows::WindowsExt as _;
use domain::works::Work;
use domain::StrId;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_shell::ShellExt;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn get_play_time_minutes(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<f32, CommandError> {
    Ok(modules
        .file_use_case()
        .get_play_time_minutes(StrId::<Work>::new(work_id))?)
}

#[tauri::command]
pub fn open_folder(path: String) -> anyhow::Result<(), CommandError> {
    let path = resolve_open_folder_path(&path)?;
    let err_msg: anyhow::Error = anyhow::anyhow!("Failed to open folder at path: {}", path);
    std::process::Command::new("explorer")
        .arg(path)
        .output()
        .map_err(|_| err_msg)?;

    Ok(())
}

fn resolve_open_folder_path(path: &str) -> anyhow::Result<String> {
    let mut resolved_path = path.to_string();
    if path.to_lowercase().ends_with(".lnk") {
        let windows = crate::infrastructure::windowsimpl::windows::Windows::new();
        if let Ok(metadatas) = windows.shell_link().get_lnk_metadatas(vec![path.to_string()]) {
            if let Some(meta) = metadatas.get(path) {
                resolved_path = meta.path.clone();
            }
        }
    }

    let resolved = std::path::Path::new(&resolved_path);
    if resolved.is_file() {
        Ok(resolved
            .parent()
            .ok_or(anyhow::anyhow!("parent not found"))?
            .to_string_lossy()
            .to_string())
    } else {
        Ok(resolved_path)
    }
}

#[tauri::command]
pub async fn get_default_import_dirs() -> anyhow::Result<Vec<String>, CommandError> {
    let user_menu = dirs::home_dir()
        .ok_or(anyhow::anyhow!("cannot got home dir"))?
        .join("AppData")
        .join("Roaming")
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .to_string_lossy()
        .to_string();

    let system_menu = "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs";

    Ok(vec![user_menu, system_menu.to_string()])
}

#[tauri::command]
pub async fn get_exe_path_by_lnk(
    _handle: AppHandle,
    filepath: String,
) -> anyhow::Result<String, CommandError> {
    if !filepath.to_lowercase().ends_with("lnk") {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "filepath is not ends with lnk"
        )));
    }

    let p = filepath.clone();
    let windows = crate::infrastructure::windowsimpl::windows::Windows::new();
    let metadatas = windows.shell_link().get_lnk_metadatas(vec![p.clone()])?;
    if let Some(meta) = metadatas.get(&p) {
        return Ok(meta.path.clone());
    } else {
        return Err(CommandError::Anyhow(anyhow::anyhow!(
            "cannot get lnk metadata"
        )));
    }
}

#[tauri::command]
pub async fn open_url(handle: AppHandle, url: String) -> anyhow::Result<(), CommandError> {
    #[allow(deprecated)]
    let shell = handle.shell();
    #[allow(deprecated)]
    shell
        .open(url, None)
        .map_err(|e| anyhow::anyhow!("Failed to open URL: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_nearest_key_and_distance(
    key: String,
    calculate_distance_kv: Vec<(String, String)>,
) -> anyhow::Result<(String, f32), CommandError> {
    let key = crate::domain::file::normalize(&key);
    let normalized_kv = calculate_distance_kv
        .into_iter()
        .map(|v| {
            (
                crate::domain::file::normalize(&v.0),
                crate::domain::file::normalize(&v.1),
            )
        })
        .collect::<Vec<(String, String)>>();

    for (comp_key, comp_value) in normalized_kv.iter() {
        if key == *comp_key {
            return Ok((comp_value.to_string(), 1.0));
        }
    }

    let mut max_distance = 0.0;
    let mut max_distance_value = None;
    for (comp_key, comp_value) in normalized_kv.into_iter() {
        let distance = crate::domain::distance::get_comparable_distance(&key, &comp_key);
        if max_distance < distance {
            max_distance = distance;
            max_distance_value = Some(comp_value);
        }
    }

    match max_distance_value {
        Some(value) => Ok((value, max_distance)),
        _ => Err(CommandError::Anyhow(anyhow::anyhow!(
            "maybe calculate_distance_kv is empty."
        ))),
    }
}
