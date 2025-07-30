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
    sync::Arc,
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
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
    Id,
};

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};

use fast_image_resize as fr;

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

pub fn get_lnk_metadatas(lnk_file_paths: Vec<&str>) -> anyhow::Result<HashMap<&str, LnkMetadata>> {
    let mut metadatas = HashMap::new();

    unsafe {
        CoInitialize(None)?;

        let mut target_path_vec: Vec<u16> = vec![0; 261];
        let target_path_slice =
            std::slice::from_raw_parts_mut(target_path_vec.as_mut_ptr(), target_path_vec.len());

        for file_path in lnk_file_paths {
            if file_path.to_lowercase().ends_with("lnk") {
                let shell_link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

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
            } else if file_path.to_lowercase().ends_with("url") {
                let icon_file = get_url_file_icon_path(file_path)?;

                metadatas.insert(
                    file_path,
                    LnkMetadata {
                        path: file_path.to_string(),
                        icon: icon_file.unwrap_or_default(),
                    },
                );
            } else {
                return Err(anyhow::anyhow!("{} is not end lnk|url", file_path));
            }
        }

        CoUninitialize();
    }
    Ok(metadatas)
}

pub fn get_most_probable_game_candidate(
    id_name_pairs: &AllGameCache,
    filepath: String,
) -> anyhow::Result<Option<(AllGameCacheOne, String)>> {
    let game_identifier = crate::usecase::game_identifier::GameIdentifierUseCase::with_default_matcher(id_name_pairs.clone());
    let candidates = game_identifier.identify_by_filepath(&filepath)?;
    Ok(candidates
        .first()
        .map(|candidate| (candidate.clone(), filepath)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_most_probable_game_candidate() {
        let res = get_most_probable_game_candidate(
            &vec![AllGameCacheOne::new(
                27123,
                "pieces/渡り鳥のソムニウム".to_string(),
            )],
            "W:\\others\\software\\Whirlpool\\pieces\\pieces.exe".to_string(),
        )
        .unwrap();
        assert!(res.is_some());
        let (pieces, _) = res.unwrap();
        assert_eq!(pieces.id, 27123);
    }
}

const ICONS_ROOT_DIR: &str = "game-icons";
pub fn get_icon_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(ICONS_ROOT_DIR);
    fs::create_dir_all(&dir).unwrap();
    Path::new(&dir)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}
pub fn save_icon_to_png(
    handle: &Arc<AppHandle>,
    file_path: &str,
    collection_element_id: &Id<CollectionElement>,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path = get_icon_path(handle, collection_element_id);

    let is_ico = file_path.to_lowercase().ends_with("ico");
    let is_exe = file_path.to_lowercase().ends_with("exe");

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

const PLAY_HISTORIES_ROOT_DIR: &str = "play-histories";
pub fn get_play_history_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(PLAY_HISTORIES_ROOT_DIR);
    fs::create_dir_all(dir).unwrap();
    Path::new(&get_save_root_abs_dir(handle))
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

fn get_lnk_start_process_script(is_run_as_admin: bool, lnk_path: &str) -> String {
    let verb = if is_run_as_admin { "-Verb RunAs" } else { "" };
    format!(
        "
    chcp 65001 | Out-Null
    filter Get-Shortcut()
    {{
        $shl  = new-object -comobject WScript.Shell
        return $shl.CreateShortcut($_)
    }}
    $shortcut_info = (\"{}\" | Get-Shortcut)

    $params = @{{
        'FilePath' = $shortcut_info.TargetPath
    }}

    # Check if WorkingDirectory exists and is not empty
    if ($null -ne $shortcut_info.WorkingDirectory -and $shortcut_info.WorkingDirectory -ne '') {{
        $params['WorkingDirectory'] = $shortcut_info.WorkingDirectory
    }}

    # Check if Arguments exists and is not empty
    if ($null -ne $shortcut_info.Arguments -and $shortcut_info.Arguments -ne '') {{
        $params['ArgumentList'] = $shortcut_info.Arguments
    }}

    $process = Start-Process @params {} -PassThru
    echo $process.Id
    ",
        lnk_path, verb
    )
}

fn get_exe_start_process_script(is_run_as_admin: bool, exe_path: &str) -> String {
    let verb = if is_run_as_admin { "-Verb RunAs" } else { "" };
    let exe_dir = std::path::Path::new(exe_path)
        .parent()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_default();
    format!(
        "
    chcp 65001 | Out-Null
    Set-Location \"{}\" | Out-Null
    $process = Start-Process \"{}\" {} -PassThru
    echo $process.Id
    ",
        exe_dir, exe_path, verb
    )
}

pub fn start_process(
    is_run_as_admin: bool,
    exe_path: Option<String>,
    lnk_path: Option<String>,
) -> anyhow::Result<Option<u32>> {
    if exe_path.is_some() && lnk_path.is_some() {
        return Err(anyhow::anyhow!(
            "Both exe_path and lnk_path are provided. Only one should be provided.",
        ));
    }

    let script: String;
    if let Some(path) = exe_path {
        script = get_exe_start_process_script(is_run_as_admin, &path);
    } else if let Some(path) = lnk_path {
        script = get_lnk_start_process_script(is_run_as_admin, &path);
    } else {
        return Err(anyhow::anyhow!(
            "Neither exe_path nor lnk_path are provided."
        ));
    }

    println!("[INFO] [start processs] script: {}", script);

    // PowerShellでスクリプトを実行
    let output = std::process::Command::new("powershell")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(&script)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "PowerShell script failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let process_id = String::from_utf8_lossy(&output.stdout);
    let process_id = process_id.trim().parse::<u32>().ok();

    Ok(process_id)
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
pub fn get_thumbnail_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(THUMBNAILS_ROOT_DIR);
    fs::create_dir_all(&dir).unwrap();
    Path::new(&dir)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}
pub fn get_origin_thumbnail_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
    src_url: &str,
) -> anyhow::Result<String> {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(THUMBNAILS_ROOT_DIR);
    let url = url::Url::parse(src_url)?;

    let filename = url
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or(anyhow::anyhow!("Failed to extract filename from URL"))?;

    fs::create_dir_all(&dir).unwrap();
    Ok(Path::new(&dir)
        .join(format!("{}-{}", collection_element_id.value, filename))
        .to_string_lossy()
        .to_string())
}
pub fn save_thumbnail(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
    src_url: String,
) -> JoinHandle<anyhow::Result<()>> {
    let collection_element_id = collection_element_id.clone();
    let handle_cloned = handle.clone();
    tauri::async_runtime::spawn(async move {
        let save_path = get_thumbnail_path(&handle_cloned, &collection_element_id);
        if !(std::path::Path::new(&save_path).exists()) && src_url != "" {
            let orig_path =
                get_origin_thumbnail_path(&handle_cloned, &collection_element_id, &src_url)?;
            save_origin_thumbnail(&src_url, &orig_path).await?;

            resize_image(&orig_path, &save_path, 400)?;
        }
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
