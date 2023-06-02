pub struct File {}

use std::path::Path;

use anyhow::Ok;
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

use crate::domain::network::ErogamescapeIDNamePair;

use super::distance::get_comparable_distance;

pub trait WString {
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

pub fn get_lnk_source_paths(lnk_file_paths: Vec<String>) -> anyhow::Result<Vec<String>> {
    let mut source_paths = vec![];

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

            source_paths.push(path);
        }

        CoUninitialize();
    }
    Ok(source_paths)
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
