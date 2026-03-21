pub struct File {}
/// .lnk/.url から取得したメタデータ
///
/// - `path`: ショートカットの遷移先
///   - .lnk の場合: ショートカットが指す実体ファイルの絶対パス（例: .exe）
///   - .url の場合: ファイル自体のパス（URL の実体ではなく .url ファイルのパス）
/// - `icon`: アイコンの取得元パス
///   - .lnk の場合: IShellLinkW::GetIconLocation で取得したアイコンファイル（例: .ico や .exe）
///   - .url の場合: INI（`IconFile=...`）から取得したアイコンファイルパス（なければ空文字）
#[derive(Debug)]
pub struct LnkMetadata {
    pub path: String,
    pub icon: String,
}

use std::{fs, io::Write, sync::Arc};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use windows::core::PCWSTR;

use crate::service::save_path_resolver::SavePathResolver;

use super::StrId;

trait WString {
    #[allow(dead_code)]
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null_terminated(&self) -> Vec<u16>;
}

impl WString for &str {
    fn to_wide(&self) -> Vec<u16> {
        self.encode_utf16().collect()
    }

    fn to_wide_null_terminated(&self) -> Vec<u16> {
        self.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[allow(dead_code)]
fn to_pcwstr(str: &str) -> PCWSTR {
    PCWSTR::from_raw(str.to_wide_null_terminated().as_ptr())
}

pub fn normalize(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch >= 'Ａ' && ch <= 'Ｚ' || ch >= 'ａ' && ch <= 'ｚ' {
            result.push((ch as u32 - 'Ａ' as u32 + 'A' as u32) as u8 as char);
        } else if ch >= '０' && ch <= '９' {
            result.push((ch as u32 - '０' as u32 + '0' as u32) as u8 as char);
        } else {
            result.push(ch);
        }
    }
    result.to_lowercase()
}

pub fn get_url_file_icon_path(url_file_path: &str) -> anyhow::Result<Option<String>> {
    let ini_contents = std::fs::read_to_string(url_file_path)?;
    Ok(get_ini_value(&ini_contents, "IconFile"))
}

fn get_ini_value(contents: &str, key: &str) -> Option<String> {
    let key_line = contents.lines().find(|&line| line.starts_with(key))?;
    let parts: Vec<&str> = key_line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some(parts[1].trim().to_string())
    } else {
        None
    }
}

// (icons dir constant is no longer used; path resolution is centralized in SavePathResolver)
pub fn save_icon_to_png(
    handle: &Arc<AppHandle>,
    resolver: &dyn SavePathResolver,
    file_path: &str,
    work_id: &StrId<crate::works::Work>,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path = resolver.icon_png_path(&work_id.value);

    let is_exe = file_path.to_lowercase().ends_with("exe");
    let is_ico = file_path.to_lowercase().ends_with("ico");

    if is_ico {
        return save_ico_to_png(file_path, &save_png_path);
    }
    if is_exe {
        return save_exe_file_png(handle, file_path, &save_png_path);
    }
    return save_default_icon(&save_png_path);
}

pub fn save_default_icon(save_png_path: &str) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_p = save_png_path.to_string();
    let handle = tauri::async_runtime::spawn(async move {
        let default_icon = include_bytes!("../../icons/notfound.png");
        let mut file = std::fs::File::create(save_p)?;
        file.write_all(default_icon)?;
        return Ok(());
    });

    Ok(handle)
}

pub fn save_ico_to_png(
    file_path: &str,
    save_png_path: &str,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    assert!(file_path.to_lowercase().ends_with("ico"));

    let p = file_path.to_string();
    let save_p = save_png_path.to_string();
    let handle = tauri::async_runtime::spawn(async move {
        match save_ico_to_png_sync(&p, &save_p) {
            Err(_) => save_default_icon(&save_p)?.await?,
            _ => Ok(()),
        }
    });

    Ok(handle)
}

pub fn save_ico_to_png_sync(file_path: &str, save_png_path: &str) -> anyhow::Result<()> {
    // Read an ICO file from disk:
    let file = std::fs::File::open(file_path)?;
    let icon_dir = ico::IconDir::read(file)?;

    let largest_entry = icon_dir
        .entries()
        .into_iter()
        .fold(None, |largest, v| match largest {
            None => {
                return Some(v);
            }
            Some(largest) => {
                if largest.width() < v.width() {
                    return Some(v);
                }
                return Some(largest);
            }
        });

    if let Some(entry) = largest_entry {
        // Decode the first entry into an image:
        let image = entry.decode()?;
        // You can get raw RGBA pixel data to pass to another image library:
        let rgba = image.rgba_data();
        assert_eq!(rgba.len(), (4 * image.width() * image.height()) as usize);
        // Alternatively, you can save the image as a PNG file:
        let file = std::fs::File::create(save_png_path)?;
        Ok(image.write_png(file)?)
    } else {
        return Err(anyhow::anyhow!("icon_dir.entries() is empty"));
    }
}

pub fn save_exe_file_png(
    handle: &Arc<AppHandle>,
    file_path: &str,
    save_png_path: &str,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path_cloned = save_png_path.to_string();
    let (mut rx, _) = handle
        .shell()
        .sidecar("extract-icon")?
        .args(vec!["48", file_path, save_png_path])
        .spawn()?;

    let handle: JoinHandle<anyhow::Result<()>> = tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(_) = event {
                return save_default_icon(&save_png_path_cloned)?.await?;
            }
            if let CommandEvent::Stderr(_) = event {
                return save_default_icon(&save_png_path_cloned)?.await?;
            }
            if let CommandEvent::Terminated(_) = event {
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("extract-icon is not terminated"))
    });

    Ok(handle)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayHistory {
    pub minutes: f32,
    pub start_date: String,
}

pub fn get_file_created_at_sync(path: &str) -> Option<DateTime<Local>> {
    let metadata = fs::metadata(path).ok();
    metadata.and_then(|meta| {
        meta.created()
            .ok()
            .and_then(|time| Some(DateTime::from(time)))
    })
}
