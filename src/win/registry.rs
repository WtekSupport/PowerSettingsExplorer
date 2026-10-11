//! Minimal advapi32 registry bindings.

use super::wide::to_wide;
use std::ffi::c_void;
use std::ptr;

pub const HKEY_LOCAL_MACHINE: isize = 0x8000_0002_u32 as isize;
pub const KEY_READ: u32 = 0x20019;
pub const KEY_WRITE: u32 = 0x20006;
pub const KEY_SET_VALUE: u32 = 0x0002;
pub const REG_SZ: u32 = 1;
pub const REG_EXPAND_SZ: u32 = 2;
pub const REG_DWORD: u32 = 4;
pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_MORE_DATA: u32 = 234;

type HKEY = isize;
type LSTATUS = u32;

#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(
        hkey: HKEY,
        sub_key: *const u16,
        options: u32,
        sam: u32,
        result: *mut HKEY,
    ) -> LSTATUS;
    fn RegCloseKey(hkey: HKEY) -> LSTATUS;
    fn RegEnumKeyExW(
        hkey: HKEY,
        index: u32,
        name: *mut u16,
        name_len: *mut u32,
        reserved: *mut u32,
        class: *mut u16,
        class_len: *mut u32,
        ft: *mut c_void,
    ) -> LSTATUS;
    fn RegQueryValueExW(
        hkey: HKEY,
        value_name: *const u16,
        reserved: *mut u32,
        reg_type: *mut u32,
        data: *mut u8,
        data_len: *mut u32,
    ) -> LSTATUS;
    fn RegSetValueExW(
        hkey: HKEY,
        value_name: *const u16,
        reserved: u32,
        reg_type: u32,
        data: *const u8,
        data_len: u32,
    ) -> LSTATUS;
    fn RegDeleteValueW(hkey: HKEY, value_name: *const u16) -> LSTATUS;
}

pub struct RegKey {
    pub handle: HKEY,
}

impl RegKey {
    pub fn open(hive: HKEY, path: &str, access: u32) -> Result<Self, u32> {
        let path_w = to_wide(path);
        let mut h: HKEY = 0;
        let st = unsafe {
            RegOpenKeyExW(hive, path_w.as_ptr(), 0, access, &mut h)
        };
        if st != ERROR_SUCCESS {
            return Err(st);
        }
        Ok(RegKey { handle: h })
    }

    pub fn enum_subkeys(&self) -> Result<Vec<String>, u32> {
        let mut out = Vec::new();
        let mut index = 0u32;
        loop {
            let mut buf = [0u16; 256];
            let mut len = buf.len() as u32;
            let st = unsafe {
                RegEnumKeyExW(
                    self.handle,
                    index,
                    buf.as_mut_ptr(),
                    &mut len,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            if st != ERROR_SUCCESS {
                break;
            }
            out.push(super::wide::from_wide(&buf[..len as usize]));
            index += 1;
        }
        Ok(out)
    }

    pub fn query_string(&self, name: &str) -> Result<Option<String>, u32> {
        let name_w = to_wide(name);
        let mut reg_type = 0u32;
        let mut data_len = 0u32;
        let st = unsafe {
            RegQueryValueExW(
                self.handle,
                name_w.as_ptr(),
                ptr::null_mut(),
                &mut reg_type,
                ptr::null_mut(),
                &mut data_len,
            )
        };
        if st == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if st != ERROR_SUCCESS && st != ERROR_MORE_DATA {
            return Err(st);
        }
        if data_len == 0 {
            return Ok(None);
        }
        if reg_type != REG_SZ && reg_type != REG_EXPAND_SZ {
            return Ok(None);
        }
        let mut buf = vec![0u8; data_len as usize];
        let mut data_len2 = data_len;
        let st2 = unsafe {
            RegQueryValueExW(
                self.handle,
                name_w.as_ptr(),
                ptr::null_mut(),
                &mut reg_type,
                buf.as_mut_ptr(),
                &mut data_len2,
            )
        };
        if st2 != ERROR_SUCCESS {
            return Err(st2);
        }
        let n_chars = (data_len2 as usize) / 2;
        if n_chars == 0 {
            return Ok(None);
        }
        let wide = unsafe {
            std::slice::from_raw_parts(buf.as_ptr() as *const u16, n_chars)
        };
        let s = super::wide::from_wide(wide).trim_end_matches('\0').trim().to_string();
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }

    pub fn query_dword(&self, name: &str) -> Result<Option<u32>, u32> {
        let name_w = to_wide(name);
        let mut reg_type = 0u32;
        let mut data = 0u32;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let st = unsafe {
            RegQueryValueExW(
                self.handle,
                name_w.as_ptr(),
                ptr::null_mut(),
                &mut reg_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_len,
            )
        };
        if st == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if st != ERROR_SUCCESS {
            return Err(st);
        }
        if reg_type != REG_DWORD {
            return Ok(None);
        }
        Ok(Some(data))
    }

    pub fn delete_value(&self, name: &str) -> Result<(), u32> {
        let name_w = to_wide(name);
        let st = unsafe { RegDeleteValueW(self.handle, name_w.as_ptr()) };
        if st == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if st != ERROR_SUCCESS {
            return Err(st);
        }
        Ok(())
    }

    pub fn set_dword(&self, name: &str, value: u32) -> Result<(), u32> {
        let name_w = to_wide(name);
        let st = unsafe {
            RegSetValueExW(
                self.handle,
                name_w.as_ptr(),
                0,
                REG_DWORD,
                &value as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            )
        };
        if st != ERROR_SUCCESS {
            return Err(st);
        }
        Ok(())
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.handle);
        }
    }
}

pub const POWER_SETTINGS_PATH: &str =
    "SYSTEM\\CurrentControlSet\\Control\\Power\\PowerSettings";
