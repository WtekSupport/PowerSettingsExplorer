//! powrprof.dll — attributes and localized friendly names.

use crate::guid::Guid;
use crate::win::wide::from_wide;
use std::ptr;

pub const POWER_ATTRIBUTE_HIDE: u32 = 1;
pub const ATTR_SHOW_IN_UI: u32 = 2;

type DWORD = u32;
type HKEY = isize;

#[link(name = "powrprof")]
extern "system" {
    fn PowerWriteSettingAttributes(
        subgroup: *const Guid,
        setting: *const Guid,
        attributes: DWORD,
    ) -> DWORD;

    fn PowerGetActiveScheme(user_root: HKEY, active_policy: *mut *mut Guid) -> DWORD;

    fn PowerReadFriendlyName(
        root_power_key: HKEY,
        scheme_guid: *const Guid,
        subgroup_guid: *const Guid,
        power_setting_guid: *const Guid,
        buffer: *mut u8,
        buffer_size: *mut DWORD,
    ) -> DWORD;
}

#[link(name = "kernel32")]
extern "system" {
    fn LocalFree(mem: isize) -> isize;
}

#[link(name = "shlwapi")]
extern "system" {
    fn SHLoadIndirectString(
        source: *const u16,
        out: *mut u16,
        out_chars: u32,
        reserved: *mut *mut std::ffi::c_void,
    ) -> i32;
}

pub fn write_setting_attributes(subgroup: &Guid, setting: &Guid, attributes: u32) -> Result<(), u32> {
    let st = unsafe {
        PowerWriteSettingAttributes(subgroup as *const Guid, setting as *const Guid, attributes)
    };
    if st != 0 {
        return Err(st);
    }
    Ok(())
}

fn active_scheme() -> Option<Guid> {
    let mut p: *mut Guid = ptr::null_mut();
    let st = unsafe { PowerGetActiveScheme(0, &mut p) };
    if st != 0 || p.is_null() {
        return None;
    }
    let g = unsafe { *p };
    unsafe {
        LocalFree(p as isize);
    }
    Some(g)
}

fn read_friendly_name_api(
    scheme: Option<&Guid>,
    subgroup: Option<&Guid>,
    setting: Option<&Guid>,
) -> Option<String> {
    let mut size = 0u32;
    let st = unsafe {
        PowerReadFriendlyName(
            0,
            scheme.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            subgroup.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            setting.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            ptr::null_mut(),
            &mut size,
        )
    };
    // ERROR_SUCCESS with size, or ERROR_MORE_DATA (234) / insufficient buffer.
    if size < 2 {
        // Retry with a reasonable buffer even if the probe failed.
        size = 512;
        if st != 0 && st != 234 {
            // continue with fixed size
        }
    }
    let mut buf = vec![0u8; size as usize];
    let mut size2 = size;
    let st2 = unsafe {
        PowerReadFriendlyName(
            0,
            scheme.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            subgroup.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            setting.map(|g| g as *const Guid).unwrap_or(ptr::null()),
            buf.as_mut_ptr(),
            &mut size2,
        )
    };
    if st2 != 0 {
        return None;
    }
    let wchar_len = (size2 as usize / 2).saturating_sub(0);
    if wchar_len == 0 {
        return None;
    }
    let wide = unsafe {
        std::slice::from_raw_parts(buf.as_ptr() as *const u16, wchar_len.max(1))
    };
    let name = from_wide(wide).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Разворачивает `@path,-id` / обычную строку в локализованное имя UI.
/// В реестре часто бывает `@dll,-id,EnglishFallback` — хвост после второго `,` отбрасываем.
pub fn resolve_indirect_string(source: &str) -> Option<String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with('@') {
        return Some(trimmed.to_string());
    }
    // Оставляем только `@path,-resourceId`
    let for_api = {
        let bytes = trimmed.as_bytes();
        let mut comma = 0usize;
        let mut cut = trimmed.len();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b',' {
                comma += 1;
                if comma == 2 {
                    cut = i;
                    break;
                }
            }
        }
        &trimmed[..cut]
    };
    let src = crate::win::wide::to_wide(for_api);
    let mut out = vec![0u16; 512];
    let hr = unsafe {
        SHLoadIndirectString(
            src.as_ptr(),
            out.as_mut_ptr(),
            out.len() as u32,
            ptr::null_mut(),
        )
    };
    if hr < 0 {
        return None;
    }
    let name = from_wide(&out).trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Имя раздела (подгруппы) на языке системы.
pub fn subgroup_display_name(subgroup: &Guid) -> Option<String> {
    if let Some(scheme) = active_scheme() {
        if let Some(n) = read_friendly_name_api(Some(&scheme), Some(subgroup), None) {
            return Some(n);
        }
    }
    // Без активной схемы — попытка только с subgroup.
    read_friendly_name_api(None, Some(subgroup), None)
}

/// Имя параметра на языке системы.
pub fn setting_display_name(subgroup: &Guid, setting: &Guid) -> Option<String> {
    if let Some(scheme) = active_scheme() {
        if let Some(n) = read_friendly_name_api(Some(&scheme), Some(subgroup), Some(setting)) {
            return Some(n);
        }
    }
    read_friendly_name_api(None, Some(subgroup), Some(setting))
}
