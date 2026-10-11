//! UI language: English or Russian (strict — no mixing).

use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub fn as_code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "eng" | "english" => Some(Lang::En),
            "ru" | "rus" | "russian" | "рус" => Some(Lang::Ru),
            _ => None,
        }
    }

    pub fn combo_index(self) -> usize {
        match self {
            Lang::Ru => 0,
            Lang::En => 1,
        }
    }

    pub fn from_combo_index(i: isize) -> Option<Self> {
        match i {
            0 => Some(Lang::Ru),
            1 => Some(Lang::En),
            _ => None,
        }
    }
}

#[link(name = "kernel32")]
extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}

pub fn detect_system_lang() -> Lang {
    let langid = unsafe { GetUserDefaultUILanguage() };
    let primary = langid & 0x3FF;
    // LANG_RUSSIAN = 0x19
    if primary == 0x19 {
        Lang::Ru
    } else {
        Lang::En
    }
}

fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("PowerSettingsExplorer");
    Some(p)
}

pub fn load_saved_lang() -> Option<Lang> {
    let mut path = config_dir()?;
    path.push("lang.txt");
    if let Ok(s) = fs::read_to_string(&path) {
        return Lang::from_code(&s);
    }
    // Migration from previous app folder name.
    let base = std::env::var_os("LOCALAPPDATA")?;
    let legacy = PathBuf::from(base)
        .join("PowerSettingsVisibility")
        .join("lang.txt");
    let s = fs::read_to_string(legacy).ok()?;
    Lang::from_code(&s)
}

pub fn save_lang(lang: Lang) {
    let Some(dir) = config_dir() else {
        return;
    };
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("lang.txt");
    let _ = fs::write(path, lang.as_code());
}

pub fn initial_lang() -> Lang {
    load_saved_lang().unwrap_or_else(detect_system_lang)
}

pub struct UiText {
    pub window_title: &'static str,
    pub root_label: &'static str,
    pub loading: &'static str,
    pub pick_setting: &'static str,
    pub list_loaded: &'static str,
    pub vis_label: &'static str,
    pub vis_on: &'static str,
    pub vis_off: &'static str,
    pub vis_auto: &'static str,
    pub lang_label: &'static str,
    pub error_title: &'static str,
    pub registry_read_fail: &'static str,
    pub registry_write_fail: &'static str,
    pub richedit_fail: &'static str,
    pub elevate_fail: &'static str,
    pub elevate_caption: &'static str,
    pub section_purpose: &'static str,
    pub section_behavior: &'static str,
    pub section_practice: &'static str,
    pub section_in_group: &'static str,
    pub section_in_list: &'static str,
    pub section_where: &'static str,
    pub section_visibility: &'static str,
    pub no_catalog_body: &'static str,
    pub root_body: &'static str,
    pub group_fallback_hint: &'static str,
    pub group_body_fmt: &'static str, // {hint}, {count}
    pub unknown_item_body: &'static str,
    pub power_setting_fallback: &'static str,
    pub preset_label: &'static str,
    pub preset_apply: &'static str,
    pub preset_save: &'static str,
    pub preset_load: &'static str,
    pub preset_builtin_all_auto: &'static str,
    pub preset_builtin_catalog_on: &'static str,
    pub preset_builtin_essential: &'static str,
    pub preset_builtin_enthusiast: &'static str,
    pub preset_confirm: &'static str, // {count}
    pub preset_result: &'static str,   // {changed}, {errors}
    pub preset_save_title: &'static str,
    pub preset_load_title: &'static str,
    pub preset_filter: &'static str,
    pub preset_saved: &'static str,
}

pub fn ui(lang: Lang) -> &'static UiText {
    match lang {
        Lang::Ru => &UI_RU,
        Lang::En => &UI_EN,
    }
}

static UI_RU: UiText = UiText {
    window_title: "Power Settings Explorer",
    root_label: "Параметры электропитания",
    loading: "Загрузка списка…",
    pick_setting: "Выберите параметр слева",
    list_loaded: "Список загружен из реестра Windows.\n\n\
Выберите пункт слева — справа появятся описание блоками и селектор видимости (Вкл / Выкл / Авто).",
    vis_label: "Показ в доп. параметрах:",
    vis_on: "Вкл",
    vis_off: "Выкл",
    vis_auto: "Авто",
    lang_label: "Язык:",
    error_title: "Ошибка",
    registry_read_fail: "Не удалось прочитать параметры из реестра (код {code}).\nЗапустите программу от имени администратора.",
    registry_write_fail: "Не удалось записать в реестр (код {code}).\nНужен запуск от администратора.",
    richedit_fail: "Не удалось загрузить riched20.dll (Rich Edit).",
    elevate_fail: "Для изменения видимости параметров нужны права администратора.\n\
Подтвердите запрос UAC или запустите программу «От имени администратора».",
    elevate_caption: "Power Settings Explorer",
    section_purpose: "Назначение",
    section_behavior: "Поведение",
    section_practice: "На практике",
    section_in_group: "В этом разделе",
    section_in_list: "В этом списке",
    section_where: "Где задать «От сети» / «От батареи»",
    section_visibility: "Видимость",
    no_catalog_body: "Назначение\nОписание для этого GUID пока не добавлено в каталог.\n\n\
Поведение\n\
Выберите параметр с известным GUID в дереве или добавьте запись в каталог приложения.",
    root_body: "Назначение\n\n\
Дерево слева — разделы и параметры из «Дополнительных параметров электропитания» Windows.\n\n\
Поведение\n\n\
Выберите раздел или параметр: справа появится описание. Для параметра доступен селектор видимости «Вкл / Выкл / Авто».\n\n\
В этом списке\n\n\
Разделов: {groups}. Параметров: {settings}.\n\n\
Видимость\n\n\
Селектор активен только для конкретного параметра, не для раздела.",
    group_fallback_hint: "Группа параметров электропитания Windows.",
    group_body_fmt: "Назначение\n\n{hint}\n\n\
В этом разделе\n\n\
Параметров в дереве: {count}. Раскройте раздел слева и выберите пункт — справа откроется подробное описание.\n\n\
Видимость\n\n\
Селектор «Вкл / Выкл / Авто» доступен только для отдельного параметра внутри раздела.",
    unknown_item_body: "Назначение\n\nВыбранный пункт дерева. Раскройте его или выберите параметр внутри раздела.",
    power_setting_fallback: "Параметр электропитания",
    preset_label: "Пресет:",
    preset_apply: "Применить",
    preset_save: "Сохранить…",
    preset_load: "Загрузить…",
    preset_builtin_all_auto: "Как Windows (Авто везде)",
    preset_builtin_catalog_on: "Документированные (вкл.)",
    preset_builtin_essential: "Базовые разделы",
    preset_builtin_enthusiast: "Энтузиаст",
    preset_confirm: "Будет пересчитана видимость для {count} параметров в дереве.\n\
Перед массовым изменением рекомендуется точка восстановления.\n\n\
Продолжить?",
    preset_result: "Готово.\nИзменено: {changed}.\nБез изменений: {skipped}.\nОшибок записи: {errors}.",
    preset_save_title: "Сохранить пресет",
    preset_load_title: "Загрузить пресет",
    preset_filter: "Пресеты (*.preset)",
    preset_saved: "Пресет сохранён.",
};

static UI_EN: UiText = UiText {
    window_title: "Power Settings Explorer",
    root_label: "Power options",
    loading: "Loading list…",
    pick_setting: "Select a setting on the left",
    list_loaded: "The list was loaded from the Windows registry.\n\n\
Select an item on the left — a sectioned description and the visibility selector (On / Off / Auto) appear on the right.",
    vis_label: "Show in advanced settings:",
    vis_on: "On",
    vis_off: "Off",
    vis_auto: "Auto",
    lang_label: "Language:",
    error_title: "Error",
    registry_read_fail: "Failed to read power settings from the registry (code {code}).\nRun the program as administrator.",
    registry_write_fail: "Failed to write to the registry (code {code}).\nAdministrator rights are required.",
    richedit_fail: "Failed to load riched20.dll (Rich Edit).",
    elevate_fail: "Administrator rights are required to change setting visibility.\n\
Approve the UAC prompt or run the program as administrator.",
    elevate_caption: "Power Settings Explorer",
    section_purpose: "Purpose",
    section_behavior: "Behavior",
    section_practice: "In practice",
    section_in_group: "In this section",
    section_in_list: "In this list",
    section_where: "Where to set On battery / Plugged in",
    section_visibility: "Visibility",
    no_catalog_body: "Purpose\nNo catalog description for this GUID yet.\n\n\
Behavior\n\
Pick a setting with a known GUID in the tree, or add an entry to the app catalog.",
    root_body: "Purpose\n\n\
The tree on the left lists sections and settings from Windows Advanced power settings.\n\n\
Behavior\n\n\
Select a section or setting to see a description on the right. For a setting, the On / Off / Auto visibility selector is available.\n\n\
In this list\n\n\
Sections: {groups}. Settings: {settings}.\n\n\
Visibility\n\n\
The selector is enabled only for an individual setting, not for a section.",
    group_fallback_hint: "A group of Windows power settings.",
    group_body_fmt: "Purpose\n\n{hint}\n\n\
In this section\n\n\
Settings in the tree: {count}. Expand the section on the left and select an item for a detailed description.\n\n\
Visibility\n\n\
The On / Off / Auto selector is available only for an individual setting inside the section.",
    unknown_item_body: "Purpose\n\nSelected tree item. Expand it or pick a setting inside the section.",
    power_setting_fallback: "Power setting",
    preset_label: "Preset:",
    preset_apply: "Apply",
    preset_save: "Save as…",
    preset_load: "Load…",
    preset_builtin_all_auto: "Windows default (Auto all)",
    preset_builtin_catalog_on: "Documented (show catalog)",
    preset_builtin_essential: "Essentials",
    preset_builtin_enthusiast: "Enthusiast",
    preset_confirm: "Visibility will be updated for {count} settings in the tree.\n\
Consider a restore point before bulk changes.\n\n\
Continue?",
    preset_result: "Done.\nChanged: {changed}.\nUnchanged: {skipped}.\nWrite errors: {errors}.",
    preset_save_title: "Save preset",
    preset_load_title: "Load preset",
    preset_filter: "Presets (*.preset)",
    preset_saved: "Preset saved.",
};

pub fn is_section_header(lang: Lang, line: &str) -> bool {
    let t = line.trim();
    let u = ui(lang);
    t == u.section_purpose
        || t == u.section_behavior
        || t == u.section_practice
        || t == u.section_in_group
        || t == u.section_in_list
        || t.starts_with(u.section_where)
        || t.starts_with(u.section_visibility)
        || t.starts_with("Видимость")
        || t.starts_with("Visibility")
        || t.starts_with("Где задать")
        || t.starts_with("Where to set")
}
