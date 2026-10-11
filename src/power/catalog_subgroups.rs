pub struct SubgroupEntry {
    pub guid: &'static str,
    pub title_ru: &'static str,
    pub hint_ru: &'static str,
}

pub static ENTRIES: &[SubgroupEntry] = &[
    SubgroupEntry {
        guid: "fea3413e-7e05-4911-9a71-700331f1c294",
        title_ru: "Общие параметры схемы",
        hint_ru: "Параметры без отдельной категории: пароль при пробуждении, тип схемы, политика устройств и сеть в сне.",
    },
    SubgroupEntry {
        guid: "0012ee47-9041-4b5d-9b77-535fba8b1442",
        title_ru: "Жёсткий диск",
        hint_ru: "Отключение диска при простое, AHCI/NVMe, лимиты мощности накопителей.",
    },
    SubgroupEntry {
        guid: "238c9fa8-0aad-41ed-83f4-97be242c8f20",
        title_ru: "Сон",
        hint_ru: "Таймауты сна и гибернации, разрешение режимов ожидания, пробуждение.",
    },
    SubgroupEntry {
        guid: "2a737441-1930-4402-8d77-b2bebba308a3",
        title_ru: "USB",
        hint_ru: "Selective suspend и управление питанием USB 3.",
    },
    SubgroupEntry {
        guid: "2e601130-5351-4d9d-8e04-252966bad054",
        title_ru: "Устойчивость при простое",
        hint_ru: "Idle Resiliency — тонкая настройка простоя процессора и I/O (Windows 10+).",
    },
    SubgroupEntry {
        guid: "4f971e89-eebd-4455-a8de-9e59040e7347",
        title_ru: "Кнопки питания и крышка",
        hint_ru: "Действия кнопки питания, сна, закрытия и открытия крышки ноутбука.",
    },
    SubgroupEntry {
        guid: "501a4d13-42af-4429-9fd1-a8218c268e20",
        title_ru: "PCI Express",
        hint_ru: "Link State Power Management для слотов PCIe (видеокарта, NVMe).",
    },
    SubgroupEntry {
        guid: "54533251-82be-4824-96c1-47b60b740d00",
        title_ru: "Процессор",
        hint_ru: "Минимум/максимум частоты, политики простоя, Turbo Boost, C-states.",
    },
    SubgroupEntry {
        guid: "7516b95f-f776-4464-8c53-06167f40cc99",
        title_ru: "Экран и видео",
        hint_ru: "Яркость, таймаут отключения дисплея, адаптивная яркость.",
    },
    SubgroupEntry {
        guid: "e73a048d-bf27-4f12-9731-8b2076e8891f",
        title_ru: "Батарея",
        hint_ru: "Уведомления о разряде, резерв, критический уровень.",
    },
    SubgroupEntry {
        guid: "9596fb26-9850-41fd-ac3e-f7c3c00afd4b",
        title_ru: "Мультимedia",
        hint_ru: "Поведение при воспроизведении видео/музыки (отключение экрана и т.п.).",
    },
    SubgroupEntry {
        guid: "de830923-a562-41af-a086-e3a2c6bad2da",
        title_ru: "Энергосбережение",
        hint_ru: "Energy Saver — пороги и режим экономии заряда.",
    },
    SubgroupEntry {
        guid: "48672f38-7a9a-4bb2-8bf8-3d85be19de4e",
        title_ru: "Прерывания (Interrupt Steering)",
        hint_ru: "Распределение IRQ между ядрами для производительности.",
    },
    SubgroupEntry {
        guid: "44f3beca-a7c0-460e-9df2-bb8b99e0cba6",
        title_ru: "Intel Graphics",
        hint_ru: "План питания встроенной графики Intel.",
    },
    SubgroupEntry {
        guid: "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c",
        title_ru: "Графика",
        hint_ru: "Дополнительные параметры GPU/графики OEM.",
    },
    SubgroupEntry {
        guid: "8619b916-e004-4dd8-9b66-dae86f806698",
        title_ru: "Presence Aware Power",
        hint_ru: "Датчики присутствия пользователя (если поддерживаются).",
    },
    SubgroupEntry {
        guid: "19cbb8fa-5279-450e-9fac-8a3d5fedd0c1",
        title_ru: "Фоновая работа",
        hint_ru: "Background activity / connected standby (редкие пункты).",
    },
    SubgroupEntry {
        guid: "0d7dbae2-4294-402a-ba8e-26777e8488cd",
        title_ru: "Производительность",
        hint_ru: "Дополнительные OEM-параметры производительности.",
    },
    SubgroupEntry {
        guid: "02f815b5-a5cf-4c84-bf20-649d1f75d3d8",
        title_ru: "Таймеры",
        hint_ru: "Служебные таймеры схемы (OEM).",
    },
    SubgroupEntry {
        guid: "48df9d60-4f68-11dc-8314-0800200c9a66",
        title_ru: "Энергосбережение (legacy)",
        hint_ru: "Устаревшая группа параметров Windows.",
    },
];

pub fn lookup(guid: &str) -> Option<&'static SubgroupEntry> {
    let g = guid.to_lowercase();
    ENTRIES.iter().find(|e| e.guid == g)
}

pub fn title_or_fallback(guid: &str) -> String {
    lookup(guid)
        .map(|e| e.title_ru.to_string())
        .unwrap_or_else(|| format!("Группа ({})", guid.to_lowercase()))
}
