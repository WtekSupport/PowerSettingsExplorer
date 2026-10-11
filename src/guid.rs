//! GUID parsing and well-known power subgroup IDs.

use std::fmt;

/// Subgroup GUID for settings that belong to no subgroup (SUB_NONE).
pub const NO_SUBGROUP_GUID: &str = "fea3413e-7e05-4911-9a71-700331f1c294";

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Guid {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().trim_matches('{').trim_matches('}');
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 5 {
            return None;
        }
        let data1 = u32::from_str_radix(parts[0], 16).ok()?;
        let data2 = u16::from_str_radix(parts[1], 16).ok()?;
        let data3 = u16::from_str_radix(parts[2], 16).ok()?;
        let mut data4 = [0u8; 8];
        let b = parts[3].len() + parts[4].len();
        if b != 16 {
            return None;
        }
        let hex: String = parts[3].chars().chain(parts[4].chars()).collect();
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            if i >= 8 {
                break;
            }
            let pair = std::str::from_utf8(chunk).ok()?;
            data4[i] = u8::from_str_radix(pair, 16).ok()?;
        }
        Some(Guid {
            data1,
            data2,
            data3,
            data4,
        })
    }

    pub fn to_canonical_lower(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_canonical_lower())
    }
}

pub fn is_guid_str(s: &str) -> bool {
    Guid::from_str(s).is_some()
}
