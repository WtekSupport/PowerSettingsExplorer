use crate::guid::{is_guid_str, Guid, NO_SUBGROUP_GUID};
use crate::power::visibility::{read_attributes, VisibilityState};
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
    pub settings: Vec<SettingRef>,
}

fn is_setting_key(key: &RegKey) -> bool {
    if key.query_dword("Attributes").ok().flatten().is_some() {
        return true;
    }
    if key.query_dword("ValueMin").ok().flatten().is_some() {
        return true;
    }
    if key.query_dword("FriendlyName").is_ok() {
        // FriendlyName is REG_EXPAND_SZ — query as dword won't work; check Attributes/ValueMin only
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
    Some(SettingRef {
        subgroup: *subgroup,
        setting,
        subgroup_str: subgroup_str.to_string(),
        setting_str: setting_str.to_lowercase(),
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
    let mut orphan_subgroup: SubgroupNode = SubgroupNode {
        guid: Guid::from_str(NO_SUBGROUP_GUID).unwrap(),
        guid_str: NO_SUBGROUP_GUID.to_string(),
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
            let mut node = SubgroupNode {
                guid: sg_guid,
                guid_str: name.to_lowercase(),
                settings: Vec::new(),
            };
            for c in setting_children {
                let path = format!("{}\\{}", sg_path, c);
                if let Some(s) = collect_setting(&sg_guid, &node.guid_str, &c, path) {
                    node.settings.push(s);
                }
            }
            node.settings.sort_by(|a, b| a.setting_str.cmp(&b.setting_str));
            subgroups.push(node);
        } else if is_setting_key(&sg_key) {
            let sg = Guid::from_str(NO_SUBGROUP_GUID).unwrap();
            if let Some(s) = collect_setting(&sg, NO_SUBGROUP_GUID, &name, sg_path) {
                orphan_subgroup.settings.push(s);
            }
        }
    }

    orphan_subgroup
        .settings
        .sort_by(|a, b| a.setting_str.cmp(&b.setting_str));
    if !orphan_subgroup.settings.is_empty() {
        subgroups.insert(0, orphan_subgroup);
    }

    subgroups.sort_by(|a, b| a.guid_str.cmp(&b.guid_str));
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
}
