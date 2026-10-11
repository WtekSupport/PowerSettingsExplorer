#![cfg_attr(not(test), windows_subsystem = "windows")]

mod backup;
mod guid;
mod i18n;
mod power;
mod win;

fn main() {
    if !win::elevate::ensure_elevated_or_exit() {
        return;
    }
    win::ui::run();
}
