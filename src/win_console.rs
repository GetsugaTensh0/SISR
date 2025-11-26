#[cfg(windows)]
mod windows_impl {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use tracing::{debug, warn};
    use windows_sys::Win32::System::Console::{
        ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, ENABLE_PROCESSED_INPUT, FreeConsole,
        GetConsoleMode, GetConsoleWindow, GetStdHandle, STD_INPUT_HANDLE, SetConsoleMode,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

    static CONSOLE_ALLOCATED: AtomicBool = AtomicBool::new(false);
    static ATTACHED_TO_PARENT: AtomicBool = AtomicBool::new(false);
    static ORIGINAL_CONSOLE_MODE: AtomicU32 = AtomicU32::new(0);

    pub fn alloc() {
        if CONSOLE_ALLOCATED.swap(true, Ordering::SeqCst) {
            return;
        }
        unsafe {
            if AttachConsole(ATTACH_PARENT_PROCESS) > 0 {
                debug!("Attached to existing console");
                ATTACHED_TO_PARENT.store(true, Ordering::SeqCst);

                // Windows shenanigans to enable Ctrl+C handling (if launched in existing shell) ¯\_(ツ)_/¯
                let stdin = GetStdHandle(STD_INPUT_HANDLE);
                let mut mode: u32 = 0;
                if GetConsoleMode(stdin, &mut mode) != 0 {
                    ORIGINAL_CONSOLE_MODE.store(mode, Ordering::SeqCst);
                    let new_mode = mode | ENABLE_PROCESSED_INPUT;
                    _ = SetConsoleMode(stdin, new_mode);
                }

                return;
            }

            if AllocConsole() > 0 {
                debug!("Allocated new console");
                let hwnd = GetConsoleWindow();
                if !hwnd.is_null() {
                    ShowWindow(hwnd, SW_HIDE);
                }
                match enable_ansi_support::enable_ansi_support() {
                    Ok(()) => {}
                    Err(_) => {
                        warn!("Failed to enable ANSI support for console");
                    }
                }
            }
        }
    }

    pub fn show() {
        unsafe {
            let hwnd = GetConsoleWindow();
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_SHOW);
            }
        }
    }

    pub fn cleanup() {
        if !ATTACHED_TO_PARENT.load(Ordering::SeqCst) {
            return;
        }
        unsafe {
            FreeConsole();
        }
    }
}

#[cfg(windows)]
pub use windows_impl::{alloc, cleanup, show};
