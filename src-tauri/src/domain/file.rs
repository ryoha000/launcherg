pub struct File {}
pub struct LnkMetadata {
    pub path: String,
    pub icon: String,
}

use std::{fs, io::Write, path::Path, time::SystemTime};

use anyhow::Ok;
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use tauri::{
    api::process::{Command, CommandEvent},
    async_runtime::JoinHandle,
};
use walkdir::WalkDir;
use windows::{
    core::{ComInterface, PCWSTR},
    Win32::{
        Storage::FileSystem::WIN32_FIND_DATAW,
        System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER},
        System::Com::{IPersistFile, STGM_READ},
        UI::Shell::{IShellLinkW, ShellLink},
    },
};

use crate::{domain::network::ErogamescapeIDNamePair, infrastructure::util::get_save_root_abs_dir};

use super::{collection::CollectionElement, distance::get_comparable_distance, Id};

trait WString {
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

fn to_pcwstr(str: &str) -> PCWSTR {
    PCWSTR::from_raw(str.to_wide_null_terminated().as_ptr())
}

const NOT_GAME_TERMS: [&str; 12] = [
    "マニュアル",
    "詳細設定",
    "はじめに",
    "サポート",
    "セーブデータ",
    "インストール",
    "アンインストール",
    "体験版",
    "install",
    "uninstall",
    "autorun",
    "削除",
];
fn not_game(filename: &str) -> bool {
    for not_game_str in NOT_GAME_TERMS {
        if filename.contains(not_game_str) {
            return true;
        }
    }
    return false;
}

const REMOVE_WORDS: [&str; 9] = [
    "を起動",
    "の起動",
    "_単独動作版",
    "「",
    "」",
    " ",
    "　",
    "ダウンロード版",
    "DL版",
];
fn remove_word(filename: &str) -> String {
    let mut result = filename.to_string();
    for word in REMOVE_WORDS.iter() {
        result = result.replace(word, "");
    }
    result
}

const IGNORE_GAME_ID: [i32; 1] = [2644];

fn get_file_name_without_extension(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            let file_name_without_extension = Path::new(file_name_str)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned());
            return file_name_without_extension;
        }
    }
    None
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

pub fn get_file_paths_by_exts(
    explorer_dir_path: String,
    filter_exts: Vec<String>,
) -> anyhow::Result<Vec<String>> {
    let mut link_file_paths = Vec::new();

    for entry in WalkDir::new(explorer_dir_path) {
        let entry = entry?;

        if entry.file_type().is_file() {
            let path = entry.path();

            if let Some(extension) = path.extension() {
                let cmp_ext = extension.to_string_lossy().to_lowercase();
                for filter_ext in filter_exts.iter() {
                    if cmp_ext == *filter_ext {
                        let path_str = path.to_string_lossy().to_string();
                        link_file_paths.push(path_str);
                    }
                }
            }
        }
    }

    Ok(link_file_paths)
}

pub fn get_lnk_metadatas(lnk_file_paths: Vec<String>) -> anyhow::Result<Vec<LnkMetadata>> {
    let mut metadatas = vec![];

    unsafe {
        CoInitialize(None)?;

        let mut target_path_vec: Vec<u16> = vec![0; 261];
        let target_path_slice =
            std::slice::from_raw_parts_mut(target_path_vec.as_mut_ptr(), target_path_vec.len());

        for file_path in lnk_file_paths.iter() {
            let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

            let persist_file: IPersistFile = ComInterface::cast(&shell_link)?;
            persist_file.Load(
                PCWSTR::from_raw(file_path.as_str().to_wide_null_terminated().as_ptr()),
                STGM_READ,
            )?;

            shell_link.GetPath(target_path_slice, &mut WIN32_FIND_DATAW::default(), 0)?;
            let path = PCWSTR::from_raw(target_path_vec.as_mut_ptr())
                .to_string()?
                .clone();
            shell_link.GetIconLocation(target_path_slice, &mut 0)?;
            let icon = PCWSTR::from_raw(target_path_vec.as_mut_ptr())
                .to_string()?
                .clone();

            metadatas.push(LnkMetadata { path, icon });
        }

        CoUninitialize();
    }
    Ok(metadatas)
}

pub fn filter_game_path(
    id_name_pairs: &Vec<ErogamescapeIDNamePair>,
    filepath: String,
) -> anyhow::Result<Option<(ErogamescapeIDNamePair, String)>> {
    let parent = Path::new(&filepath)
        .parent()
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        })
        .ok_or(anyhow::anyhow!("can not get parent"))?;

    let filename = get_file_name_without_extension(&filepath)
        .ok_or(anyhow::anyhow!("can not get filename"))?;
    let filename = normalize(&filename);
    if not_game(&filename) {
        return Ok(None);
    }
    let filename = remove_word(&filename);

    let is_skip_filename_check = filename == "game" || filename == "start";

    // 編集距離は最小でも0.8欲しい
    let mut max_distance_value = 0.8;
    let mut max_distance_pair = None;
    for pair in id_name_pairs.iter() {
        let mut is_ignore = false;
        for ignore_id in IGNORE_GAME_ID {
            if pair.id == ignore_id {
                is_ignore = true;
            }
        }
        if is_ignore {
            continue;
        }

        let mut val: f32 = 0.0;
        if !is_skip_filename_check {
            val = val.max(get_comparable_distance(&filename, &pair.gamename));
        }

        val = val.max(get_comparable_distance(&parent, &pair.gamename));
        if val > max_distance_value {
            max_distance_value = val;
            max_distance_pair = Some(pair.clone());
        }
    }

    if let Some(pair) = max_distance_pair {
        return Ok(Some((pair, filepath)));
    }

    for pair in id_name_pairs.iter() {
        if filename.len() > 5 && pair.gamename.contains(&filename) {
            return Ok(Some((pair.clone(), filepath)));
        }
    }
    Ok(None)
}

const ICONS_ROOT_DIR: &str = "game-icons";
pub fn get_icon_path(collection_element_id: &Id<CollectionElement>) -> String {
    let dir = Path::new(&get_save_root_abs_dir()).join(ICONS_ROOT_DIR);
    fs::create_dir_all(dir).unwrap();
    Path::new(&get_save_root_abs_dir())
        .join(ICONS_ROOT_DIR)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}
pub fn save_icon_to_png(
    file_path: &str,
    collection_element_id: &Id<CollectionElement>,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path = get_icon_path(collection_element_id);

    let is_ico = file_path.to_lowercase().ends_with("ico");
    let is_exe = file_path.to_lowercase().ends_with("exe");

    if is_ico {
        return save_ico_to_png(file_path, &save_png_path);
    }
    if is_exe {
        return save_exe_file_png(file_path, &save_png_path);
    }
    return save_default_icon(&save_png_path);
}

pub fn save_default_icon(save_png_path: &str) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_p = save_png_path.to_string();
    let handle = tauri::async_runtime::spawn(async move {
        let default_icon = include_bytes!("..\\..\\icons\\notfound.png");
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
    Ok(image::io::Reader::open(file_path)?
        .decode()?
        .save(save_png_path)?)
}

pub fn save_exe_file_png(
    file_path: &str,
    save_png_path: &str,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path_cloned = save_png_path.to_string();
    let (mut rx, _) = Command::new_sidecar("extract-icon")?
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

const PLAY_HISTORIES_ROOT_DIR: &str = "play-histories";
pub fn get_play_history_path(collection_element_id: &Id<CollectionElement>) -> String {
    let dir = Path::new(&get_save_root_abs_dir()).join(PLAY_HISTORIES_ROOT_DIR);
    fs::create_dir_all(dir).unwrap();
    Path::new(&get_save_root_abs_dir())
        .join(PLAY_HISTORIES_ROOT_DIR)
        .join(format!("{}.jsonl", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayHistory {
    pub minutes: f32,
    pub start_date: String,
}

pub fn start_process(
    is_run_as_admin: bool,
    path: &str,
    play_history_path: &str,
) -> anyhow::Result<()> {
    let verb = match is_run_as_admin {
        true => "-Verb RunAs",
        false => "",
    };

    let script = format!(
        r#"
    $startDate = Get-Date -Format "yyyy/MM/dd HH:mm:ss"
    $executionTime = Measure-Command {{ Start-Process "{}" {} -Wait }}
    $totalMinutes = $executionTime.TotalMinutes
    $jsonObject = New-Object PSObject -Property @{{
        minutes = $totalMinutes
        startDate = $startDate
    }}
    $json = $jsonObject | ConvertTo-Json -Compress
    Add-Content -Path "{}" -Value $json
    "#,
        path, verb, play_history_path
    );

    std::process::Command::new("powershell.exe")
        .arg("-Command")
        .arg(&script)
        .spawn()?;

    Ok(())
}

pub fn get_file_created_at_sync(path: &str) -> Option<DateTime<Local>> {
    let metadata = fs::metadata(path).ok();
    metadata.and_then(|meta| {
        meta.created()
            .ok()
            .and_then(|time| Some(DateTime::from(time)))
    })
}

pub fn get_file_created_at(path: &str) -> JoinHandle<Option<DateTime<Local>>> {
    let path = path.to_string();
    tauri::async_runtime::spawn(async move { get_file_created_at_sync(&path) })
}
