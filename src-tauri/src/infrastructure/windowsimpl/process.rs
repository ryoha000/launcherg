use async_trait::async_trait;
use windows::Win32::{
    Foundation::MAX_PATH,
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW},
};

use super::{screenshot::take, windows::WindowsImpl};
use crate::domain::{process::Process, windows::process::ProcessWindows};

#[async_trait]
impl ProcessWindows for WindowsImpl<Process> {
    fn save_screenshot_by_process_id(&self, process_id: u32, filepath: &str) -> anyhow::Result<()> {
        take::take_screenshot_by_process_id(process_id, filepath)
    }
    fn save_top_window_screenshot(&self, filepath: &str) -> anyhow::Result<()> {
        take::take_screenshot_by_top_window(filepath)
    }
    fn get_top_window_name(&self) -> anyhow::Result<String> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0 == 0 {
            return Err(anyhow::anyhow!("cannot get top window"));
        }
        let mut window_text = vec![0u16; MAX_PATH as usize];
        unsafe { GetWindowTextW(hwnd, &mut window_text.as_mut_slice()) };
        Ok(String::from_utf16_lossy(&window_text)
            .trim_end_matches('\0')
            .to_string())
    }
}
