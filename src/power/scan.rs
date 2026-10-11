use crate::guid::{is_guid_str, Guid, NO_SUBGROUP_GUID};
use crate::power::visibility::{read_attributes, VisibilityState};
use crate::win::powrprof::{
    resolve_indirect_string, setting_display_name, subgroup_display_name,
};
use crate::win::registry::{
    RegKey, POWER_SETTINGS_PATH, HKEY_LOCAL_MACHINE, KEY_READ,
};

const SKIP_CHILD: &[&str] = &["0", "1", "DefaultPowerSchemeValues"];

#[derive(Clone)]
pub struct SettingRef {
    pub subgroup: Guid,
    pub setting: Guid,
    pub subgroup_str: String,
    pub setting_str: String,
    /// Имя из Windows (язык ОС). В UI предпочитаем каталог выбранного языка.
    pub system_name: String,
    /// HKLM-relative path without prefix, e.g. PowerSettings\...\...
    pub registry_path: String,
    pub attributes: Option<u32>,
    pub visibility: VisibilityState,
    pub has_value_min: bool,
}

#[derive(Clone)]
pub struct SubgroupNode {
    pub guid: Guid,
    pub guid_str: String,
    /// Имя раздела из Windows (язык ОС). В UI предпочитаем каталог выбранного языка.
    pub system_name: String,
    pub settings: Vec<SettingRef>,
}

fn is_setting_key(key: &RegKey) -> bool {
    if key.query_dword("Attributes").ok().flatten().is_some() {
        return true;
    }
    if key.query_dword("ValueMin").ok().flatten().is_some() {
        return true;
    }
    false
}

fn child_is_setting(sub_path: &str) -> bool {
    RegKey::open(HKEY_LOCAL_MACHINE, sub_path, KEY_READ)
        .map(|k| {
            k.query_dword("Attributes").ok().flatten().is_some()
                || k.query_dword("ValueMin").ok().flatten().is_some()
        })
        .unwrap_or(false)
}

fn registry_friendly_name(registry_path: &str) -> Option<String> {
    let key = RegKey::open(HKEY_LOCAL_MACHINE, registry_path, KEY_READ).ok()?;
    let raw = key.query_string("FriendlyName").ok().flatten()?;
    resolve_indirect_string(&raw).or_else(|| {
        let t = raw.trim();
        if t.is_empty() || t.starts_with('@') {
            None
        } else {
            Some(t.to_string())
        }
    })
}

fn resolve_subgroup_name(guid: &Guid, guid_str: &str, registry_path: &str) -> String {
    subgroup_display_name(guid)
        .or_else(|| registry_friendly_name(registry_path))
        .unwrap_or_else(|| format!("({})", &guid_str[..8.min(guid_str.len())]))
}

fn resolve_setting_name(
    subgroup: &Guid,
    setting: &Guid,
    setting_str: &str,
    registry_path: &str,
) -> String {
    setting_display_name(subgroup, setting)
        .or_else(|| registry_friendly_name(registry_path))
        .unwrap_or_else(|| setting_str.to_string())
}

fn collect_setting(
    subgroup: &Guid,
    subgroup_str: &str,
    setting_str: &str,
    registry_path: String,
) -> Option<SettingRef> {
    let setting = Guid::from_str(setting_str)?;
    let attrs = read_attributes(&registry_path).ok().flatten();
    let has_value_min = RegKey::open(HKEY_LOCAL_MACHINE, &registry_path, KEY_READ)
        .ok()
        .and_then(|k| k.query_dword("ValueMin").ok().flatten())
        .is_some();
    let setting_str_l = setting_str.to_lowercase();
    let system_name =
        resolve_setting_name(subgroup, &setting, &setting_str_l, &registry_path);
    Some(SettingRef {
        subgroup: *subgroup,
        setting,
        subgroup_str: subgroup_str.to_string(),
        setting_str: setting_str_l,
        system_name,
        registry_path,
        visibility: VisibilityState::from_attributes(attrs),
        attributes: attrs,
        has_value_min,
    })
}

pub fn scan_all() -> Result<Vec<SubgroupNode>, u32> {
    let root = RegKey::open(HKEY_LOCAL_MACHINE, POWER_SETTINGS_PATH, KEY_READ)?;
    let top_keys = root.enum_subkeys()?;
    let mut subgroups: Vec<SubgroupNode> = Vec::new();
    let orphan_guid = Guid::from_str(NO_SUBGROUP_GUID).unwrap();
    let orphan_path = format!("{}\\{}", POWER_SETTINGS_PATH, NO_SUBGROUP_GUID);
    let mut orphan_subgroup: SubgroupNode = SubgroupNode {
        guid: orphan_guid,
        guid_str: NO_SUBGROUP_GUID.to_string(),
        system_name: resolve_subgroup_name(&orphan_guid, NO_SUBGROUP_GUID, &orphan_path),
        settings: Vec::new(),
    };

    for name in top_keys {
        if !is_guid_str(&name) {
            continue;
        }
        let sg_path = format!("{}\\{}", POWER_SETTINGS_PATH, name);
        let sg_key = match RegKey::open(HKEY_LOCAL_MACHINE, &sg_path, KEY_READ) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let children: Vec<String> = sg_key
            .enum_subkeys()?
            .into_iter()
            .filter(|c| is_guid_str(c) && !SKIP_CHILD.contains(&c.as_str()))
            .collect();

        let mut setting_children = Vec::new();
        for c in &children {
            let path = format!("{}\\{}", sg_path, c);
            if child_is_setting(&path) {
                setting_children.push(c.clone());
            }
        }

        if !setting_children.is_empty() {
            let sg_guid = Guid::from_str(&name).unwrap();
            let guid_str = name.to_lowercase();
            let system_name = resolve_subgroup_name(&sg_guid, &guid_str, &sg_path);
            let mut node = SubgroupNode {
                guid: sg_guid,
                guid_str,
                system_name,
                settings: Vec::new(),
            };
            for c in setting_children {
                let path = format!("{}\\{}", sg_path, c);
                if let Some(s) = collect_setting(&sg_guid, &node.guid_str, &c, path) {
                    node.settings.push(s);
                }
            }
            node.settings
                .sort_by(|a, b| a.system_name.to_lowercase().cmp(&b.system_name.to_lowercase()));
            subgroups.push(node);
        } else if is_setting_key(&sg_key) {
            let sg = Guid::from_str(NO_SUBGROUP_GUID).unwrap();
            if let Some(s) = collect_setting(&sg, NO_SUBGROUP_GUID, &name, sg_path) {
                orphan_subgroup.settings.push(s);
            }
        }
    }

    orphan_subgroup.settings.sort_by(|a, b| {
        a.system_name
            .to_lowercase()
            .cmp(&b.system_name.to_lowercase())
    });
    if !orphan_subgroup.settings.is_empty() {
        subgroups.insert(0, orphan_subgroup);
    }

    subgroups.sort_by(|a, b| {
        a.system_name
            .to_lowercase()
            .cmp(&b.system_name.to_lowercase())
    });
    Ok(subgroups)
}

pub fn all_settings(nodes: &[SubgroupNode]) -> Vec<&SettingRef> {
    nodes.iter().flat_map(|n| n.settings.iter()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_finds_many_settings() {
        let nodes = scan_all().expect("registry scan");
        let count: usize = nodes.iter().map(|n| n.settings.len()).sum();
        assert!(
            count >= 140,
            "expected ~155 settings on this machine, got {count}"
        );
    }

    #[test]
    fn device_idle_policy_present() {
        let nodes = scan_all().expect("registry scan");
        let found = nodes.iter().flat_map(|n| n.settings.iter()).any(|s| {
            s.setting_str == "4faab71a-92e5-4726-b531-224559672d19"
        });
        assert!(found, "Device idle policy GUID should be enumerated");
    }

    #[test]
    fn display_names_are_localized() {
        let nodes = scan_all().expect("registry scan");
        let with_names = nodes
            .iter()
            .filter(|n| !n.system_name.starts_with('('))
            .count();
        assert!(with_names > 0, "expected at least one localized subgroup name");
        let setting_named = nodes
            .iter()
            .flat_map(|n| n.settings.iter())
            .filter(|s| s.system_name != s.setting_str)
            .count();
        assert!(
            setting_named > 50,
            "expected many settings with FriendlyName, got {setting_named}"
        );
    }
}
