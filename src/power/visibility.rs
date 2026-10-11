use crate::guid::Guid;
use crate::win::powrprof::{self, ATTR_SHOW_IN_UI, POWER_ATTRIBUTE_HIDE};
use crate::win::registry::{RegKey, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WRITE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityMode {
    /// Явно показывать в доп. параметрах (Attributes без HIDE, обычно 2).
    On,
    /// Скрыть (POWER_ATTRIBUTE_HIDE).
    Off,
    /// Как по умолчанию Windows — удалить Attributes из реестра.
    Auto,
}

impl VisibilityMode {
    pub fn combo_index(self) -> usize {
        match self {
            VisibilityMode::On => 0,
            VisibilityMode::Off => 1,
            VisibilityMode::Auto => 2,
        }
    }

    pub fn from_combo_index(i: isize) -> Option<Self> {
        match i {
            0 => Some(VisibilityMode::On),
            1 => Some(VisibilityMode::Off),
            2 => Some(VisibilityMode::Auto),
            _ => None,
        }
    }
}

pub fn mode_from_attributes(attrs: Option<u32>) -> VisibilityMode {
    match attrs {
        None => VisibilityMode::Auto,
        Some(a) if a & POWER_ATTRIBUTE_HIDE != 0 => VisibilityMode::Off,
        _ => VisibilityMode::On,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityState {
    /// No Attributes value — shown by default in advanced UI.
    DefaultVisible,
    Hidden,
    Shown,
    /// Attributes present but not clearly hide/show (other flags).
    Other(u32),
}

impl VisibilityState {
    pub fn from_attributes(attrs: Option<u32>) -> Self {
        match attrs {
            None => VisibilityState::DefaultVisible,
            Some(a) if a & POWER_ATTRIBUTE_HIDE != 0 => VisibilityState::Hidden,
            Some(2) | Some(0) => VisibilityState::Shown,
            Some(a) => VisibilityState::Other(a),
        }
    }

    pub fn visible_in_advanced_ui(self) -> bool {
        !matches!(self, VisibilityState::Hidden)
    }

    pub fn label_ru(&self) -> &'static str {
        match self {
            VisibilityState::DefaultVisible => {
                "По умолчанию отображается (в реестре нет значения Attributes)"
            }
            VisibilityState::Hidden => "Скрыто в «Дополнительных параметрах»",
            VisibilityState::Shown => "Отображается в «Дополнительных параметрах»",
            VisibilityState::Other(_) => "Особый режим видимости (нестандартный Attributes)",
        }
    }
}

pub fn read_attributes(registry_rel_path: &str) -> Result<Option<u32>, u32> {
    let key = RegKey::open(HKEY_LOCAL_MACHINE, registry_rel_path, KEY_READ)?;
    key.query_dword("Attributes")
}

pub fn set_hidden(subgroup: &Guid, setting: &Guid, registry_rel_path: &str) -> Result<(), u32> {
    if powrprof::write_setting_attributes(subgroup, setting, POWER_ATTRIBUTE_HIDE).is_err() {
        let key = RegKey::open(
            HKEY_LOCAL_MACHINE,
            registry_rel_path,
            KEY_WRITE | KEY_SET_VALUE,
        )?;
        key.set_dword("Attributes", POWER_ATTRIBUTE_HIDE)?;
    }
    Ok(())
}

pub fn set_shown(subgroup: &Guid, setting: &Guid, registry_rel_path: &str) -> Result<(), u32> {
    if let Err(_e) = powrprof::write_setting_attributes(subgroup, setting, ATTR_SHOW_IN_UI) {
        let key = RegKey::open(
            HKEY_LOCAL_MACHINE,
            registry_rel_path,
            KEY_WRITE | KEY_SET_VALUE,
        )?;
        key.set_dword("Attributes", ATTR_SHOW_IN_UI)?;
    }
    Ok(())
}

pub fn set_auto(
    _subgroup: &Guid,
    _setting: &Guid,
    registry_rel_path: &str,
) -> Result<(), u32> {
    let key = RegKey::open(
        HKEY_LOCAL_MACHINE,
        registry_rel_path,
        KEY_WRITE | KEY_SET_VALUE,
    )?;
    key.delete_value("Attributes")
}

pub fn apply_mode(
    subgroup: &Guid,
    setting: &Guid,
    registry_rel_path: &str,
    mode: VisibilityMode,
) -> Result<(), u32> {
    match mode {
        VisibilityMode::On => set_shown(subgroup, setting, registry_rel_path),
        VisibilityMode::Off => set_hidden(subgroup, setting, registry_rel_path),
        VisibilityMode::Auto => set_auto(subgroup, setting, registry_rel_path),
    }
}
