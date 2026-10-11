use crate::i18n::Lang;

pub struct SubgroupEntry {
    pub guid: &'static str,
    pub title_ru: &'static str,
    pub title_en: &'static str,
    pub hint_ru: &'static str,
    pub hint_en: &'static str,
}

impl SubgroupEntry {
    pub fn title(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Ru => self.title_ru,
            Lang::En => self.title_en,
        }
    }

    pub fn hint(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Ru => self.hint_ru,
            Lang::En => self.hint_en,
        }
    }
}

pub static ENTRIES: &[SubgroupEntry] = &[
    SubgroupEntry {
        guid: "fea3413e-7e05-4911-9a71-700331f1c294",
        title_ru: "Общие параметры схемы",
        title_en: "General",
        hint_ru: "Параметры без отдельной категории: пароль при пробуждении, тип схемы, политика устройств и сеть в сне.",
        hint_en: "Settings without a dedicated category: require a password on wake, power scheme type, device idle policy, and network connectivity in standby.",
    },
    SubgroupEntry {
        guid: "0012ee47-9041-4b5d-9b77-535fba8b1442",
        title_ru: "Жёсткий диск",
        title_en: "Hard disk",
        hint_ru: "Отключение диска при простое, AHCI/NVMe, лимиты мощности накопителей.",
        hint_en: "Disk idle power-down, AHCI/NVMe link power, and storage power limits.",
    },
    SubgroupEntry {
        guid: "238c9fa8-0aad-41ed-83f4-97be242c8f20",
        title_ru: "Сон",
        title_en: "Sleep",
        hint_ru: "Таймауты сна и гибернации, разрешение режимов ожидания, пробуждение.",
        hint_en: "Sleep and hibernate timeouts, allowed standby states, and wake behavior.",
    },
    SubgroupEntry {
        guid: "2a737441-1930-4402-8d77-b2bebba308a3",
        title_ru: "USB",
        title_en: "USB",
        hint_ru: "Selective suspend и управление питанием USB 3.",
        hint_en: "USB selective suspend and USB 3 link power management.",
    },
    SubgroupEntry {
        guid: "2e601130-5351-4d9d-8e04-252966bad054",
        title_ru: "Устойчивость при простое",
        title_en: "Idle Resiliency",
        hint_ru: "Idle Resiliency — тонкая настройка простоя процессора и I/O (Windows 10+).",
        hint_en: "Idle Resiliency — fine-tuning of processor idle and I/O coalescing (Windows 10+).",
    },
    SubgroupEntry {
        guid: "4f971e89-eebd-4455-a8de-9e59040e7347",
        title_ru: "Кнопки питания и крышка",
        title_en: "Power buttons and lid",
        hint_ru: "Действия кнопки питания, сна, закрытия и открытия крышки ноутбука.",
        hint_en: "Actions for the power button, sleep button, and laptop lid open/close.",
    },
    SubgroupEntry {
        guid: "501a4d13-42af-4429-9fd1-a8218c268e20",
        title_ru: "PCI Express",
        title_en: "PCI Express",
        hint_ru: "Link State Power Management для слотов PCIe (видеокарта, NVMe).",
        hint_en: "Link State Power Management for PCIe devices (GPU, NVMe, controllers).",
    },
    SubgroupEntry {
        guid: "54533251-82be-4824-96c1-47b60b740d00",
        title_ru: "Процессор",
        title_en: "Processor",
        hint_ru: "Минимум/максимум частоты, политики простоя, Turbo Boost, C-states.",
        hint_en: "Min/max frequency, idle policies, Turbo Boost, and C-states.",
    },
    SubgroupEntry {
        guid: "7516b95f-f776-4464-8c53-06167f40cc99",
        title_ru: "Экран и видео",
        title_en: "Display",
        hint_ru: "Яркость, таймаут отключения дисплея, адаптивная яркость.",
        hint_en: "Brightness, display turn-off timeout, and adaptive brightness.",
    },
    SubgroupEntry {
        guid: "e73a048d-bf27-4f12-9731-8b2076e8891f",
        title_ru: "Батарея",
        title_en: "Battery",
        hint_ru: "Уведомления о разряде, резерв, критический уровень.",
        hint_en: "Low-battery notifications, reserve level, and critical battery actions.",
    },
    SubgroupEntry {
        guid: "9596fb26-9850-41fd-ac3e-f7c3c00afd4b",
        title_ru: "Мультимедиа",
        title_en: "Multimedia settings",
        hint_ru: "Поведение при воспроизведении видео/музыки (отключение экрана и т.п.).",
        hint_en: "Behavior during video/music playback (display power-down, etc.).",
    },
    SubgroupEntry {
        guid: "de830923-a562-41af-a086-e3a2c6bad2da",
        title_ru: "Энергосбережение",
        title_en: "Energy Saver",
        hint_ru: "Energy Saver — пороги и режим экономии заряда.",
        hint_en: "Energy Saver thresholds and battery-saving mode.",
    },
    SubgroupEntry {
        guid: "48672f38-7a9a-4bb2-8bf8-3d85be19de4e",
        title_ru: "Прерывания (Interrupt Steering)",
        title_en: "Interrupt Steering",
        hint_ru: "Распределение IRQ между ядрами для производительности.",
        hint_en: "How hardware IRQs are distributed across CPU cores.",
    },
    SubgroupEntry {
        guid: "44f3beca-a7c0-460e-9df2-bb8b99e0cba6",
        title_ru: "Intel Graphics",
        title_en: "Intel Graphics",
        hint_ru: "План питания встроенной графики Intel.",
        hint_en: "Power plan for Intel integrated graphics.",
    },
    SubgroupEntry {
        guid: "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c",
        title_ru: "Графика",
        title_en: "Graphics",
        hint_ru: "Дополнительные параметры GPU/графики OEM.",
        hint_en: "Additional OEM GPU/graphics power settings.",
    },
    SubgroupEntry {
        guid: "8619b916-e004-4dd8-9b66-dae86f806698",
        title_ru: "Датчики присутствия",
        title_en: "Presence Aware Power",
        hint_ru: "Датчики присутствия пользователя (если поддерживаются).",
        hint_en: "User presence sensors (when supported by the hardware).",
    },
    SubgroupEntry {
        guid: "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
        title_ru: "Фоновая работа",
        title_en: "Background settings",
        hint_ru: "Background activity / connected standby (редкие пункты).",
        hint_en: "Background activity / connected standby (less common settings).",
    },
    SubgroupEntry {
        guid: "0d7dbae2-4294-402a-ba8e-26777e8488cd",
        title_ru: "Производительность",
        title_en: "Performance",
        hint_ru: "Дополнительные OEM-параметры производительности.",
        hint_en: "Additional OEM performance-related power settings.",
    },
    SubgroupEntry {
        guid: "02f815b5-a5cf-4c84-bf20-649d1f75d3d8",
        title_ru: "Таймеры",
        title_en: "Timers",
        hint_ru: "Служебные таймеры схемы (OEM).",
        hint_en: "Scheme service timers (OEM).",
    },
    SubgroupEntry {
        guid: "48df9d60-4f68-11dc-8314-0800200c9a66",
        title_ru: "Энергосбережение (legacy)",
        title_en: "Energy Saver (legacy)",
        hint_ru: "Устаревшая группа параметров Windows.",
        hint_en: "Legacy Windows power-setting group.",
    },
];

pub fn lookup(guid: &str) -> Option<&'static SubgroupEntry> {
    let g = guid.to_lowercase();
    ENTRIES.iter().find(|e| e.guid == g)
}

pub fn title_or_fallback(guid: &str, lang: Lang) -> String {
    if let Some(e) = lookup(guid) {
        return e.title(lang).to_string();
    }
    match lang {
        Lang::Ru => format!("Группа ({})", guid.to_lowercase()),
        Lang::En => format!("Group ({})", guid.to_lowercase()),
    }
}
