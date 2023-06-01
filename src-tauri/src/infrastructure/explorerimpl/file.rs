use async_trait::async_trait;
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

use super::explorer::ExplorerImpl;
use crate::domain::{
    explorer::file::FileExplorer,
    file::{File, LnkMetadata},
};

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

#[async_trait]
impl FileExplorer for ExplorerImpl<File> {
    async fn get_file_paths_by_exts(
        &self,
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

    async fn get_lnk_source_paths(
        &self,
        lnk_file_paths: Vec<String>,
    ) -> anyhow::Result<Vec<LnkMetadata>> {
        let mut source_paths = vec![];

        unsafe {
            CoInitialize(None)?;

            for file_path in lnk_file_paths.iter() {
                let shell_link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

                let mut target_path_vec: Vec<u16> = vec![0; 261];
                let target_path_slice = std::slice::from_raw_parts_mut(
                    target_path_vec.as_mut_ptr(),
                    target_path_vec.len(),
                );

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
                let icon_path = PCWSTR::from_raw(target_path_vec.as_mut_ptr()).to_string()?;

                source_paths.push(LnkMetadata { path, icon_path });
            }

            CoUninitialize();
        }
        Ok(source_paths)
    }
}

#[cfg(test)]
mod tests {
    use tauri::async_runtime::block_on;

    use super::*;

    #[test]
    fn test_get_files() {
        let file_explorer: ExplorerImpl<File> = ExplorerImpl::new();
        let files = block_on(file_explorer.get_file_paths_by_exts(
            "C:\\Users\\ryoha\\Desktop\\妹！せいかつ～ファンタジー～1.3.3".to_string(),
            vec!["lnk".to_string(), "exe".to_string()],
        ))
        .unwrap();
        assert_eq!(files, vec!["C:\\Users\\ryoha\\Desktop\\妹！せいかつ～ファンタジー～1.3.3\\Config.exe".to_string(), "C:\\Users\\ryoha\\Desktop\\妹！せいかつ～ファンタジー～1.3.3\\Game.exe".to_string(), "C:\\Users\\ryoha\\Desktop\\妹！せいかつ～ファンタジー～1.3.3\\神様のような君へ Extended Edition.lnk".to_string()]);
    }

    #[test]
    fn test_get_src() {
        let file_explorer: ExplorerImpl<File> = ExplorerImpl::new();
        let files = block_on(file_explorer.get_file_paths_by_exts(
            "C:\\Users\\ryoha\\Desktop\\既プレイゲーム".to_string(),
            vec!["lnk".to_string()],
        ))
        .unwrap();

        let srcs = block_on(file_explorer.get_lnk_source_paths(files)).unwrap();

        println!("{:#?}", srcs);

        use std::path::Path;
        for src in srcs.iter() {
            let path = Path::new(&src.path);
            assert_eq!(path.exists(), true);

            let path = Path::new(&src.icon_path);
            assert_eq!(path.exists(), true);
        }
    }
}
