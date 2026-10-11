//! Restart elevated (UAC) when the process is not admin.

use crate::i18n;
use crate::win::wide::to_wide;
use std::ptr;

const SW_SHOW: i32 = 5;

#[link(name = "shell32")]
extern "system" {
    fn IsUserAnAdmin() -> i32;
    fn ShellExecuteW(
        hwnd: isize,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_cmd: i32,
    ) -> isize;
}

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, typ: u32) -> i32;
}

const MB_OK: u32 = 0;
const MB_ICONWARNING: u32 = 0x30;

/// `true` — текущий процесс может показывать UI; `false` — выйти (запущена копия с UAC или отказ).
pub fn ensure_elevated_or_exit() -> bool {
    unsafe {
        if IsUserAnAdmin() != 0 {
            return true;
        }
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let exe_w = to_wide(&exe.to_string_lossy());
    let op = to_wide("runas");

    let rc = unsafe {
        ShellExecuteW(
            0,
            op.as_ptr(),
            exe_w.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        )
    };

    if rc as isize > 32 {
        return false;
    }

    let t = i18n::ui(i18n::initial_lang());
    let msg = to_wide(t.elevate_fail);
    let cap = to_wide(t.elevate_caption);
    unsafe {
        MessageBoxW(0, msg.as_ptr(), cap.as_ptr(), MB_OK | MB_ICONWARNING);
    }
    false
}
