use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

const FIVEM_PROCESSES: &[&str] = &[
    "fivem.exe",
    "fivem_b2802_gtaprocess.exe",
    "fivem_gtaprocess.exe",
    "fivem_dumpserver.exe",
];

pub fn kill_fivem_processes() -> Result<u32, String> {
    let mut killed = 0u32;

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| format!("Failed to create snapshot: {}", e))?;

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );
                let name_lower = name.to_lowercase();

                if FIVEM_PROCESSES.iter().any(|p| name_lower == *p) {
                    if let Ok(handle) =
                        OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID)
                    {
                        if TerminateProcess(handle, 1).is_ok() {
                            killed += 1;
                            log::info!("Killed process: {} (PID: {})", name, entry.th32ProcessID);
                        }
                        let _ = CloseHandle(handle);
                    }
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
    }

    Ok(killed)
}

/// Launch FiveM via explorer.exe to bypass FiveM's cmd/shell blocking.
/// FiveM detects and blocks launches from cmd.exe/powershell.exe,
/// but explorer.exe is trusted as a normal user shell action.
pub fn launch_fivem() -> Result<(), String> {
    let url: Vec<u16> = "fivem://connect/cfx.re/join/aaex7k\0"
        .encode_utf16()
        .collect();

    // Primary method: explorer.exe (bypasses FiveM anti-automation)
    let result = std::process::Command::new("explorer.exe")
        .arg("fivem://connect/cfx.re/join/aaex7k")
        .spawn();

    match result {
        Ok(_) => {
            log::info!("FiveM launched via explorer.exe");
            return Ok(());
        }
        Err(e) => {
            log::warn!("explorer.exe failed ({}), falling back to ShellExecuteW", e);
        }
    }

    // Fallback: ShellExecuteW (Win32 API, also bypasses cmd blocking)
    unsafe {
        let open: Vec<u16> = "open\0".encode_utf16().collect();
        let ret = ShellExecuteW(
            None,
            PCWSTR(open.as_ptr()),
            PCWSTR(url.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );

        // ShellExecuteW returns > 32 on success
        let code = ret.0 as isize;
        if code > 32 {
            log::info!("FiveM launched via ShellExecuteW");
            Ok(())
        } else {
            Err(format!(
                "ShellExecuteW failed with code {} — FiveM may not be installed",
                code
            ))
        }
    }
}
