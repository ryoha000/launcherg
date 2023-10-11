use sysinfo::{PidExt, ProcessExt, SystemExt};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT, TRUE},
    UI::WindowsAndMessaging::{GetWindowInfo, GetWindowThreadProcessId, WINDOWINFO},
};

struct FindWindowData {
    process_id: u32,
    hwnds: Vec<HWND>,
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut FindWindowData);
    let process_id = &mut 0;
    GetWindowThreadProcessId(hwnd, Some(process_id));
    if *process_id == data.process_id {
        data.hwnds.push(hwnd);
    }
    TRUE
}

fn find_window_handles_by_process_id(process_id: u32) -> anyhow::Result<Vec<HWND>> {
    let mut data: FindWindowData = FindWindowData {
        process_id,
        hwnds: vec![],
    };

    unsafe {
        windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut data as *mut _ as isize),
        )?;
    }
    Ok(data.hwnds)
}

struct ProcessIdCandidate {
    started: Option<u32>,
    children: Vec<u32>,
}
fn find_candidate_process_ids_by_started(started_process_id: u32) -> ProcessIdCandidate {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let mut process_id_candidates: ProcessIdCandidate = ProcessIdCandidate {
        started: None,
        children: vec![],
    };

    for (process_id, process) in system.processes() {
        if process_id.as_u32() == started_process_id {
            process_id_candidates.started = Some(started_process_id);
        } else if process.parent().map(|p| p.as_u32()).unwrap_or(0) == started_process_id {
            process_id_candidates.children.push(process_id.as_u32());
        }
    }
    process_id_candidates
}

fn get_window_info(hwnd: HWND) -> anyhow::Result<RECT> {
    let mut wi = WINDOWINFO::default();
    wi.cbSize = std::mem::size_of::<WINDOWINFO>() as u32;
    unsafe { GetWindowInfo(hwnd, &mut wi)? };

    Ok(wi.rcClient)
}

fn find_capturable_window_by_pid(pid: u32) -> anyhow::Result<HWND> {
    find_window_handles_by_process_id(pid).and_then(|hwnds| {
        hwnds
            .into_iter()
            .find(|hwnd| {
                get_window_info(*hwnd)
                    .map(|rect| {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        println!("process_id: {}, width: {}, height: {}", pid, width, height);
                        if width > 400 && height > 200 {
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("No capture window found"))
    })
}

pub fn find_capture_hwnd(started_process_id: u32) -> anyhow::Result<HWND> {
    let process_id_candidates = find_candidate_process_ids_by_started(started_process_id);
    if let Some(process_id) = process_id_candidates.started {
        if let Ok(hwnd) = find_capturable_window_by_pid(process_id) {
            println!("Found capture window by started pid: {}", process_id);
            return Ok(hwnd);
        }
    }
    for process_id in process_id_candidates.children {
        if let Ok(hwnd) = find_capturable_window_by_pid(process_id) {
            println!("Found capture window by pid: {}", process_id);
            return Ok(hwnd);
        }
    }
    Err(anyhow::anyhow!("No capture window found"))
}
