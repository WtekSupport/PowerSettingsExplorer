//! Visibility presets — bulk On / Off / Auto for all scanned settings.

use crate::guid::Guid;
use crate::i18n::Lang;
use crate::power::catalog;
use crate::power::scan::{all_settings, SettingRef, SubgroupNode};
use crate::power::visibility::{apply_mode, mode_from_attributes, VisibilityMode};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const PRESET_FILE_HEADER: &str = "# Power Settings Explorer preset v1\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinPresetId {
    AllAuto,
    CatalogOn,
    Essential,
    Enthusiast,
}

impl BuiltinPresetId {
    pub fn display_name(self, lang: Lang) -> &'static str {
        let t = crate::i18n::ui(lang);
        match self {
            BuiltinPresetId::AllAuto => t.preset_builtin_all_auto,
            BuiltinPresetId::CatalogOn => t.preset_builtin_catalog_on,
            BuiltinPresetId::Essential => t.preset_builtin_essential,
            BuiltinPresetId::Enthusiast => t.preset_builtin_enthusiast,
        }
    }

    pub const ALL: [BuiltinPresetId; 4] = [
        BuiltinPresetId::AllAuto,
        BuiltinPresetId::CatalogOn,
        BuiltinPresetId::Essential,
        BuiltinPresetId::Enthusiast,
    ];

}

#[derive(Clone, Debug)]
pub enum PresetRule {
    InCatalog {
        mode: VisibilityMode,
    },
    NotInCatalog {
        mode: VisibilityMode,
    },
    Subgroup {
        subgroup: Guid,
        mode: VisibilityMode,
    },
    Unnamed {
        mode: VisibilityMode,
    },
}

#[derive(Clone, Debug)]
pub struct PresetDefinition {
    pub name: String,
    pub default: VisibilityMode,
    pub rules: Vec<PresetRule>,
    /// Explicit per-setting overrides (file snapshots / user presets).
    pub explicit: HashMap<(Guid, Guid), VisibilityMode>,
}

impl PresetDefinition {
    pub fn builtin(id: BuiltinPresetId) -> Self {
        match id {
            BuiltinPresetId::AllAuto => PresetDefinition {
                name: "all_auto".to_string(),
                default: VisibilityMode::Auto,
                rules: vec![],
                explicit: HashMap::new(),
            },
            BuiltinPresetId::CatalogOn => PresetDefinition {
                name: "catalog_on".to_string(),
                default: VisibilityMode::Off,
                rules: vec![
                    PresetRule::InCatalog {
                        mode: VisibilityMode::On,
                    },
                    PresetRule::NotInCatalog {
                        mode: VisibilityMode::Off,
                    },
                ],
                explicit: HashMap::new(),
            },
            BuiltinPresetId::Essential => PresetDefinition {
                name: "essential".to_string(),
                default: VisibilityMode::Off,
                rules: std::iter::once(PresetRule::Unnamed {
                    mode: VisibilityMode::Off,
                })
                .chain(essential_subgroup_rules(VisibilityMode::On))
                .collect(),
                explicit: HashMap::new(),
            },
            BuiltinPresetId::Enthusiast => PresetDefinition {
                name: "enthusiast".to_string(),
                default: VisibilityMode::Off,
                rules: std::iter::once(PresetRule::Unnamed {
                    mode: VisibilityMode::Off,
                })
                .chain(enthusiast_subgroup_rules(VisibilityMode::On))
                .collect(),
                explicit: HashMap::new(),
            },
        }
    }
}

fn guid(s: &str) -> Guid {
    Guid::from_str(s).expect("static guid")
}

fn essential_subgroup_rules(mode: VisibilityMode) -> Vec<PresetRule> {
    [
        "238c9fa8-0aad-41ed-83f4-97be242c8f20", // sleep
        "7516b95f-f776-4464-8c53-06167f40cc99", // display
        "e73a048d-bf27-4f12-9731-8b2076e8891f", // battery
        "4f971e89-eebd-4455-a8de-9e59040e7347", // power buttons
    ]
    .iter()
    .map(|g| PresetRule::Subgroup {
        subgroup: guid(g),
        mode,
    })
    .collect()
}

fn enthusiast_subgroup_rules(mode: VisibilityMode) -> Vec<PresetRule> {
    [
        "54533251-82be-4824-96c1-47b60b740d00", // processor
        "0012ee47-9041-4b5d-9b77-535fba8b1442", // disk
        "501a4d13-42af-4429-9fd1-a8218c268e20", // PCI
        "2a737441-1930-4402-8d77-b2bebba308a3", // USB
        "238c9fa8-0aad-41ed-83f4-97be242c8f20", // sleep
        "44f3beca-a7c0-460e-9df2-bb8b99e0cba6", // Intel graphics
        "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c", // graphics
    ]
    .iter()
    .map(|g| PresetRule::Subgroup {
        subgroup: guid(g),
        mode,
    })
    .collect()
}

pub fn is_unnamed_setting(s: &SettingRef) -> bool {
    let n = s.system_name.trim();
    if n.is_empty() {
        return true;
    }
    if n.starts_with('(') && n.ends_with(')') && n.len() >= 10 {
        return true;
    }
    false
}

pub fn resolve_mode(s: &SettingRef, preset: &PresetDefinition) -> VisibilityMode {
    let key = (s.subgroup, s.setting);
    if let Some(&m) = preset.explicit.get(&key) {
        return m;
    }
    for rule in &preset.rules {
        if rule_matches(s, rule) {
            return rule_mode(rule);
        }
    }
    preset.default
}

fn rule_mode(rule: &PresetRule) -> VisibilityMode {
    match rule {
        PresetRule::InCatalog { mode }
        | PresetRule::NotInCatalog { mode }
        | PresetRule::Subgroup { mode, .. }
        | PresetRule::Unnamed { mode } => *mode,
    }
}

fn rule_matches(s: &SettingRef, rule: &PresetRule) -> bool {
    match rule {
        PresetRule::InCatalog { .. } => catalog::lookup(&s.setting_str).is_some(),
        PresetRule::NotInCatalog { .. } => catalog::lookup(&s.setting_str).is_none(),
        PresetRule::Subgroup { subgroup, .. } => s.subgroup == *subgroup,
        PresetRule::Unnamed { .. } => is_unnamed_setting(s),
    }
}

#[derive(Clone, Debug, Default)]
pub struct ApplyResult {
    pub changed: usize,
    pub skipped: usize,
    pub errors: Vec<(String, u32)>,
}

pub fn apply_preset(nodes: &[SubgroupNode], preset: &PresetDefinition) -> ApplyResult {
    let mut result = ApplyResult::default();
    for s in all_settings(nodes) {
        let target = resolve_mode(s, preset);
        let current = mode_from_attributes(s.attributes);
        if target == current {
            result.skipped += 1;
            continue;
        }
        if let Err(code) = apply_mode(&s.subgroup, &s.setting, &s.registry_path, target) {
            result.errors.push((s.setting_str.clone(), code));
            continue;
        }
        result.changed += 1;
    }
    result
}

pub fn presets_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("PowerSettingsExplorer");
    p.push("presets");
    Some(p)
}

pub fn backups_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("PowerSettingsExplorer");
    p.push("backups");
    Some(p)
}

pub fn list_user_presets() -> Vec<PathBuf> {
    let Some(dir) = presets_dir() else {
        return Vec::new();
    };
    let Ok(read) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "preset"))
        .collect();
    files.sort_by(|a, b| {
        let am = fs::metadata(a).and_then(|m| m.modified()).ok();
        let bm = fs::metadata(b).and_then(|m| m.modified()).ok();
        bm.cmp(&am)
    });
    files
}

pub fn mode_to_token(m: VisibilityMode) -> &'static str {
    match m {
        VisibilityMode::On => "on",
        VisibilityMode::Off => "off",
        VisibilityMode::Auto => "auto",
    }
}

pub fn mode_from_token(s: &str) -> Option<VisibilityMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "on" => Some(VisibilityMode::On),
        "off" => Some(VisibilityMode::Off),
        "auto" => Some(VisibilityMode::Auto),
        _ => None,
    }
}

pub fn load_preset_file(path: &Path) -> Result<PresetDefinition, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_preset_text(&text)
}

pub fn parse_preset_text(text: &str) -> Result<PresetDefinition, String> {
    let mut name = String::from("custom");
    let mut default = VisibilityMode::Auto;
    let mut explicit = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(v) = line.strip_prefix("name=") {
            name = v.trim().to_string();
            continue;
        }
        if let Some(v) = line.strip_prefix("default=") {
            default = mode_from_token(v).ok_or_else(|| format!("bad default: {v}"))?;
            continue;
        }
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 3 {
            continue;
        }
        let subgroup = Guid::from_str(parts[0]).ok_or_else(|| format!("bad subgroup: {}", parts[0]))?;
        let setting = Guid::from_str(parts[1]).ok_or_else(|| format!("bad setting: {}", parts[1]))?;
        let mode = mode_from_token(parts[2]).ok_or_else(|| format!("bad mode: {}", parts[2]))?;
        explicit.insert((subgroup, setting), mode);
    }

    Ok(PresetDefinition {
        name,
        default,
        rules: vec![],
        explicit,
    })
}

pub fn save_preset_file(path: &Path, name: &str, nodes: &[SubgroupNode]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut f = fs::File::create(path).map_err(|e| e.to_string())?;
    f.write_all(PRESET_FILE_HEADER.as_bytes())
        .map_err(|e| e.to_string())?;
    write!(f, "name={name}\n").map_err(|e| e.to_string())?;
    write!(f, "default=auto\n").map_err(|e| e.to_string())?;
    for s in all_settings(nodes) {
        let mode = mode_from_attributes(s.attributes);
        writeln!(
            f,
            "{};{};{}",
            s.subgroup_str.to_ascii_lowercase(),
            s.setting_str.to_ascii_lowercase(),
            mode_to_token(mode)
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::power::visibility::VisibilityState;

    fn sample_setting(sub: &str, set: &str, name: &str) -> SettingRef {
        SettingRef {
            subgroup: Guid::from_str(sub).unwrap(),
            setting: Guid::from_str(set).unwrap(),
            subgroup_str: sub.to_string(),
            setting_str: set.to_string(),
            system_name: name.to_string(),
            registry_path: String::new(),
            attributes: None,
            visibility: VisibilityState::DefaultVisible,
            has_value_min: false,
        }
    }

    #[test]
    fn parse_preset_file_default_and_explicit() {
        let text = r#"
# Power Settings Explorer preset v1
name=Test
default=off
0012ee47-9041-4b5d-9b77-535fba8b1442;0b2d69d7-a2a1-449c-9680-f91c70521c60;on
"#;
        let p = parse_preset_text(text).unwrap();
        assert_eq!(p.name, "Test");
        assert_eq!(p.default, VisibilityMode::Off);
        let s = sample_setting(
            "0012ee47-9041-4b5d-9b77-535fba8b1442",
            "0b2d69d7-a2a1-449c-9680-f91c70521c60",
            "AHCI",
        );
        assert_eq!(resolve_mode(&s, &p), VisibilityMode::On);
        let other = sample_setting(
            "238c9fa8-0aad-41ed-83f4-97be242c8f20",
            "25dfa149-5dd1-4736-b5ab-e8a37b5b8187",
            "Sleep",
        );
        assert_eq!(resolve_mode(&other, &p), VisibilityMode::Off);
    }

    #[test]
    fn catalog_on_rules() {
        let p = PresetDefinition::builtin(BuiltinPresetId::CatalogOn);
        let in_cat = sample_setting(
            "0012ee47-9041-4b5d-9b77-535fba8b1442",
            "0b2d69d7-a2a1-449c-9680-f91c70521c60",
            "AHCI",
        );
        assert_eq!(resolve_mode(&in_cat, &p), VisibilityMode::On);
    }

    #[test]
    fn all_auto_is_default() {
        let p = PresetDefinition::builtin(BuiltinPresetId::AllAuto);
        let s = sample_setting(
            "0012ee47-9041-4b5d-9b77-535fba8b1442",
            "0b2d69d7-a2a1-449c-9680-f91c70521c60",
            "x",
        );
        assert_eq!(resolve_mode(&s, &p), VisibilityMode::Auto);
    }

    #[test]
    fn unnamed_detection() {
        let s = sample_setting(
            "0012ee47-9041-4b5d-9b77-535fba8b1442",
            "0b2d69d7-a2a1-449c-9680-f91c70521c60",
            "(0b2d69d7)",
        );
        assert!(is_unnamed_setting(&s));
    }
}
