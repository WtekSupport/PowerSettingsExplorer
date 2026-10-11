use crate::guid::Guid;
use crate::power::scan::{all_settings, SubgroupNode};
use crate::power::visibility::{set_hidden, set_shown};
use std::fs;
use std::io::{BufRead, Write};

const HEADER: &str = "# power-settings-visibility backup v1\n";

pub fn export_backup(path: &str, nodes: &[SubgroupNode]) -> std::io::Result<usize> {
    let mut f = fs::File::create(path)?;
    f.write_all(HEADER.as_bytes())?;
    let mut n = 0;
    for s in all_settings(nodes) {
        let attrs = s.attributes.map(|a| a.to_string()).unwrap_or_else(|| "-".to_string());
        let line = format!(
            "{};{};{}\n",
            s.subgroup_str, s.setting_str, attrs
        );
        f.write_all(line.as_bytes())?;
        n += 1;
    }
    Ok(n)
}

pub fn import_backup(path: &str) -> Result<usize, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut n = 0;
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 3 {
            continue;
        }
        let subgroup = Guid::from_str(parts[0]).ok_or("bad subgroup guid")?;
        let setting = Guid::from_str(parts[1]).ok_or("bad setting guid")?;
        let reg_path = if parts[0].eq_ignore_ascii_case(crate::guid::NO_SUBGROUP_GUID) {
            format!(
                "{}\\{}",
                crate::win::registry::POWER_SETTINGS_PATH,
                parts[1]
            )
        } else {
            format!(
                "{}\\{}\\{}",
                crate::win::registry::POWER_SETTINGS_PATH,
                parts[0],
                parts[1]
            )
        };
        let attrs = parts[2];
        if attrs == "-" {
            set_shown(&subgroup, &setting, &reg_path).map_err(|e| format!("write {e}"))?;
        } else {
            let v: u32 = attrs.parse().map_err(|_| "bad attrs")?;
            if v & 1 != 0 {
                set_hidden(&subgroup, &setting, &reg_path).map_err(|e| format!("write {e}"))?;
            } else {
                set_shown(&subgroup, &setting, &reg_path).map_err(|e| format!("write {e}"))?;
            }
        }
        n += 1;
    }
    Ok(n)
}
