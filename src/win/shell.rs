//! ShellExecute for opening power control panel.

use super::wide::to_wide;

const SW_SHOW: i32 = 5;

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_cmd: i32,
    ) -> isize;
}

pub fn open_power_options() {
    let op = to_wide("open");
    let file = to_wide("powercfg.cpl");
    unsafe {
        ShellExecuteW(
            0,
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        );
    }
}
