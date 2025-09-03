use domain::windows::shell_link::{ShellLink, CreateShortcutRequest};
use windows::{
    core::{ComInterface, PCWSTR},
    Win32::{
        Storage::FileSystem::WIN32_FIND_DATAW,
        System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER},
        System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE},
        System::Com::{IPersistFile, STGM_READ},
        UI::Shell::{IShellLinkW, ShellLink as ShellLinkCom, ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW},
        UI::WindowsAndMessaging::SW_SHOWNORMAL,
    },
};

#[derive(Default)]
pub struct ShellLinkImpl;

impl ShellLinkImpl { pub fn new() -> Self { Self::default() } }

trait WString { fn to_wide_null_terminated(&self) -> Vec<u16>; }
impl WString for &str { fn to_wide_null_terminated(&self) -> Vec<u16> { self.encode_utf16().chain(std::iter::once(0)).collect() } }

impl ShellLink for ShellLinkImpl {
    fn create_bulk(&self, items: Vec<CreateShortcutRequest>) -> anyhow::Result<()> {
        if items.is_empty() { return Ok(()); }
        unsafe {
            CoInitialize(None)?;
            for item in items.iter() {
                let shell_link: IShellLinkW = CoCreateInstance(&ShellLinkCom, None, CLSCTX_INPROC_SERVER)?;
                shell_link.SetPath(PCWSTR::from_raw((&item.target_path as &str).to_wide_null_terminated().as_ptr()))?;
                if let Some(wd) = &item.working_dir { shell_link.SetWorkingDirectory(PCWSTR::from_raw((wd.as_str()).to_wide_null_terminated().as_ptr()))?; }
                if let Some(args) = &item.arguments { shell_link.SetArguments(PCWSTR::from_raw((args.as_str()).to_wide_null_terminated().as_ptr()))?; }
                if let Some(icon) = &item.icon_path { shell_link.SetIconLocation(PCWSTR::from_raw((icon.as_str()).to_wide_null_terminated().as_ptr()), 0)?; }
                let persist_file: IPersistFile = ComInterface::cast(&shell_link)?;
                persist_file.Save(PCWSTR::from_raw((&item.dest_lnk_path as &str).to_wide_null_terminated().as_ptr()), true)?;
            }
            CoUninitialize();
        }
        Ok(())
    }

    fn get_lnk_metadatas(&self, lnk_file_paths: Vec<String>) -> anyhow::Result<std::collections::HashMap<String, domain::file::LnkMetadata>> {
        let mut metadatas: std::collections::HashMap<String, domain::file::LnkMetadata> = std::collections::HashMap::new();
        unsafe {
            CoInitialize(None)?;
            let mut target_path_vec: Vec<u16> = vec![0; 261];
            let target_path_slice = std::slice::from_raw_parts_mut(target_path_vec.as_mut_ptr(), target_path_vec.len());
            for file_path in lnk_file_paths.into_iter() {
                if file_path.to_lowercase().ends_with("lnk") {
                    let shell_link: IShellLinkW = CoCreateInstance(&ShellLinkCom, None, CLSCTX_INPROC_SERVER)?;
                    let persist_file: IPersistFile = ComInterface::cast(&shell_link)?;
                    persist_file.Load(PCWSTR::from_raw(file_path.as_str().to_wide_null_terminated().as_ptr()), STGM_READ)?;
                    shell_link.GetPath(target_path_slice, &mut WIN32_FIND_DATAW::default(), 0)?;
                    let path = PCWSTR::from_raw(target_path_vec.as_mut_ptr()).to_string()?.clone();
                    shell_link.GetIconLocation(target_path_slice, &mut 0)?;
                    let icon = PCWSTR::from_raw(target_path_vec.as_mut_ptr()).to_string()?.clone();
                    metadatas.insert(file_path, domain::file::LnkMetadata { path, icon });
                } else if file_path.to_lowercase().ends_with("url") {
                    let icon_file: Option<String> = domain::file::get_url_file_icon_path(&file_path)?;
                    metadatas.insert(file_path.clone(), domain::file::LnkMetadata { path: file_path, icon: icon_file.unwrap_or_default() });
                } else {
                    return Err(anyhow::anyhow!("{} is not end lnk|url", file_path));
                }
            }
            CoUninitialize();
        }
        Ok(metadatas)
    }

    fn execute_lnk<'a>(&self, lnk_path: &'a str, is_run_as_admin: bool) -> anyhow::Result<Option<u32>> {
        unsafe {
            CoInitialize(None)?;
            let mut sei = SHELLEXECUTEINFOW::default();
            let lp_file_w = (&lnk_path).to_wide_null_terminated();
            sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            sei.fMask = SEE_MASK_NOCLOSEPROCESS;
            sei.lpFile = PCWSTR::from_raw(lp_file_w.as_ptr());
            sei.nShow = SW_SHOWNORMAL.0 as i32;
            if is_run_as_admin { sei.lpVerb = PCWSTR::from_raw((&"runas").to_wide_null_terminated().as_ptr()); }

            ShellExecuteExW(&mut sei as *mut SHELLEXECUTEINFOW)
                .map_err(|_| anyhow::anyhow!("ShellExecuteExW failed"))?;

            let h_process = sei.hProcess;
            if h_process.0 == 0 { CoUninitialize(); return Ok(None); }
            let _ = WaitForSingleObject(h_process, INFINITE);
            let mut exit_code: u32 = 0;
            GetExitCodeProcess(h_process, &mut exit_code as *mut u32)
                .map_err(|_| anyhow::anyhow!("GetExitCodeProcess failed"))?;
            CoUninitialize();
            Ok(Some(exit_code))
        }
    }
}


