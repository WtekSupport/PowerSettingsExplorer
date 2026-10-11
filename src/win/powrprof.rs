//! powrprof.dll — PowerWriteSettingAttributes.

use crate::guid::Guid;

pub const POWER_ATTRIBUTE_HIDE: u32 = 1;
pub const ATTR_SHOW_IN_UI: u32 = 2;

type DWORD = u32;

#[link(name = "powrprof")]
extern "system" {
    fn PowerWriteSettingAttributes(
        subgroup: *const Guid,
        setting: *const Guid,
        attributes: DWORD,
    ) -> DWORD;
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
