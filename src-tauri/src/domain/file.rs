pub struct File {}
pub struct LnkMetadata {
    pub path: String,
    pub icon: String,
}

use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
    num::NonZeroU32,
    path::Path,
};

use chrono::{DateTime, Local};
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

use crate::infrastructure::util::get_save_root_abs_dir;

use super::{
    all_game_cache::{AllGameCache, AllGameCacheOne},
    collection::CollectionElement,
    distance::get_comparable_distance,
    Id,
};

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};

use fast_image_resize as fr;

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

const NOT_GAME_TERMS: [&str; 16] = [
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
    "license",
    "ライセンス",
    "公式サイト",
    "ホームページ",
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

pub fn get_file_name_without_extension(file_path: &str) -> Option<String> {
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

pub fn get_lnk_metadatas(lnk_file_paths: Vec<&str>) -> anyhow::Result<HashMap<&str, LnkMetadata>> {
    let mut metadatas = HashMap::new();

    unsafe {
        CoInitialize(None)?;

        let mut target_path_vec: Vec<u16> = vec![0; 261];
        let target_path_slice =
            std::slice::from_raw_parts_mut(target_path_vec.as_mut_ptr(), target_path_vec.len());

        for file_path in lnk_file_paths {
            let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

            let persist_file: IPersistFile = ComInterface::cast(&shell_link)?;
            persist_file.Load(
                PCWSTR::from_raw(file_path.to_wide_null_terminated().as_ptr()),
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

            metadatas.insert(file_path, LnkMetadata { path, icon });
        }

        CoUninitialize();
    }
    Ok(metadatas)
}

pub fn filter_game_path(
    id_name_pairs: &AllGameCache,
    filepath: String,
) -> anyhow::Result<Option<(AllGameCacheOne, String)>> {
    let candidates: AllGameCache =
        get_game_candidates_by_exe_path(id_name_pairs, &filepath, 0.8, 1)?;
    if let Some(candidate) = candidates.get(0) {
        return Ok(Some((candidate.clone(), filepath)));
    }
    return Ok(None);
}

pub fn get_game_candidates_by_exe_path(
    id_name_pairs: &AllGameCache,
    filepath: &str,
    threshould: f32,
    candidate_limit: usize,
) -> anyhow::Result<AllGameCache> {
    let parent = Path::new(&filepath)
        .parent()
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        })
        .ok_or(anyhow::anyhow!("can not get parent"))?;

    let filename: String =
        get_file_name_without_extension(filepath).ok_or(anyhow::anyhow!("can not get filename"))?;
    let filename = normalize(&filename);
    if not_game(&filename) {
        return Ok(vec![]);
    }
    let filename = remove_word(&filename);

    let is_skip_filename_check = filename == "game" || filename == "start";

    let mut distance_pairs = vec![];
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
        if val > threshould {
            distance_pairs.push((pair.clone(), val));
        }
    }

    if distance_pairs.len() == 0 {
        for pair in id_name_pairs.iter() {
            if filename.len() > 5 && pair.gamename.contains(&filename) {
                distance_pairs.push((pair.clone(), filename.len() as f32));
            }
            if parent.len() > 5 && pair.gamename.contains(&parent) {
                distance_pairs.push((pair.clone(), parent.len() as f32));
            }
        }
    }

    distance_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut res = vec![];
    for (pair, _) in distance_pairs {
        if res.len() < candidate_limit {
            res.push(pair)
        }
    }
    Ok(res)
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

// const TEMP_SCRIPTS_ROOT_DIR: &str = "temp-scripts";
// pub fn get_temp_script_dir_path() -> String {
//     let dir = Path::new(&get_save_root_abs_dir()).join(TEMP_SCRIPTS_ROOT_DIR);
//     fs::create_dir_all(dir).unwrap();
//     Path::new(&get_save_root_abs_dir())
//         .join(PLAY_HISTORIES_ROOT_DIR)
//         .join(format!("{}.jsonl", collection_element_id.value))
//         .to_string_lossy()
//         .to_string()
// }

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
    let path = path.to_string();
    let play_history_path = play_history_path.to_string();
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let start_date = Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        let mut child = match is_run_as_admin {
            true => std::process::Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(r#"Start-Process "{}" -Verb RunAs"#, path),
                ])
                .spawn()
                .unwrap(),
            false => std::process::Command::new(&path).spawn().unwrap(),
        };
        child.wait().unwrap();

        let duration = start.elapsed();
        let minutes = duration.as_secs_f64() / 60.0;

        let history = PlayHistory {
            minutes: minutes as f32,
            start_date,
        };

        // JSONにシリアライズ
        let serialized = serde_json::to_string(&history).unwrap();

        // ファイルに追記
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(play_history_path)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", serialized) {
            eprintln!("Couldn't write to file: {}", e);
        }
    });

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

const THUMBNAILS_ROOT_DIR: &str = "thumbnails";
pub fn get_thumbnail_path(collection_element_id: &Id<CollectionElement>) -> String {
    let dir = Path::new(&get_save_root_abs_dir()).join(THUMBNAILS_ROOT_DIR);
    fs::create_dir_all(dir).unwrap();
    Path::new(&get_save_root_abs_dir())
        .join(THUMBNAILS_ROOT_DIR)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}
pub fn get_origin_thumbnail_path(
    collection_element_id: &Id<CollectionElement>,
    src_url: &str,
) -> anyhow::Result<String> {
    let dir = Path::new(&get_save_root_abs_dir()).join(THUMBNAILS_ROOT_DIR);
    let url = url::Url::parse(src_url)?;

    let filename = url
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or(anyhow::anyhow!("Failed to extract filename from URL"))?;

    fs::create_dir_all(dir).unwrap();
    Ok(Path::new(&get_save_root_abs_dir())
        .join(THUMBNAILS_ROOT_DIR)
        .join(format!("{}-{}", collection_element_id.value, filename))
        .to_string_lossy()
        .to_string())
}
pub fn save_thumbnail(
    collection_element_id: &Id<CollectionElement>,
    src_url: String,
) -> JoinHandle<anyhow::Result<()>> {
    let collection_element_id = collection_element_id.clone();
    tauri::async_runtime::spawn(async move {
        let orig_path = get_origin_thumbnail_path(&collection_element_id, &src_url)?;
        save_origin_thumbnail(&src_url, &orig_path).await?;

        resize_image(&orig_path, &get_thumbnail_path(&collection_element_id), 400)?;

        Ok(())
    })
}

async fn save_origin_thumbnail(url: &str, save_path: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let response = client.get(url).send().await?;

    let mut output_file = std::fs::File::create(save_path)?;
    let bytes = response.bytes().await?;

    output_file.write_all(&bytes)?;

    Ok(())
}

fn resize_image(src: &str, dst: &str, dst_width_px: u32) -> anyhow::Result<()> {
    // Read source image from file
    let img = ImageReader::open(src)?.decode()?;

    let width = NonZeroU32::new(img.width()).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
    let height = NonZeroU32::new(img.height()).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )?;

    // Multiple RGB channels of source image by alpha channel
    // (not required for the Nearest algorithm)
    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

    // Create container for data of destination image
    let dst_width =
        NonZeroU32::new(dst_width_px).ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;
    let dst_height =
        NonZeroU32::new((height.get() as f32 / width.get() as f32 * dst_width_px as f32) as u32)
            .ok_or(anyhow::anyhow!("failed NonZeroU32::new"))?;

    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));
    resizer.resize(&src_image.view(), &mut dst_view)?;

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;

    let mut result_buf = BufWriter::new(fs::File::create(&dst)?);

    // Write destination image as PNG-file
    PngEncoder::new(&mut result_buf).write_image(
        dst_image.buffer(),
        dst_width.get(),
        dst_height.get(),
        ColorType::Rgba8,
    )?;

    Ok(())
}
