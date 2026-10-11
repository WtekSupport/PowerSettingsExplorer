# -*- coding: utf-8 -*-
import csv
import re
from pathlib import Path

from catalog_manual_ru import MANUAL

ROOT = Path(__file__).resolve().parent.parent
CSV = ROOT / "settings_export.csv"
OUT = ROOT / "src" / "power" / "catalog.rs"

SUBGROUP_RU = {
    "fea3413e-7e05-4911-9a71-700331f1c294": "общие параметры схемы",
    "0012ee47-9041-4b5d-9b77-535fba8b1442": "жёсткий диск",
    "238c9fa8-0aad-41ed-83f4-97be242c8f20": "сон и гибернация",
    "2a737441-1930-4402-8d77-b2bebba308a3": "USB",
    "2e601130-5351-4d9d-8e04-252966bad054": "устойчивость при простое",
    "4f971e89-eebd-4455-a8de-9e59040e7347": "кнопки питания и крышка",
    "501a4d13-42af-4429-9fd1-a8218c268e20": "PCI Express",
    "54533251-82be-4824-96c1-47b60b740d00": "процессор",
    "7516b95f-f776-4464-8c53-06167f40cc99": "экран и видео",
    "e73a048d-bf27-4f12-9731-8b2076e8891f": "батарея",
    "9596fb26-9850-41fd-ac3e-f7c3c00afd4b": "мультимedia",
    "de830923-a562-41af-a086-e3a2c6bad2da": "энергосбережение",
    "8619b916-e004-4dd8-9b66-dae86f806698": "датчики присутствия",
    "48672f38-7a9a-4bb2-8bf8-3d85be19de4e": "прерывания (IRQ)",
    "44f3beca-a7c0-460e-9df2-bb8b99e0cba6": "графика Intel",
    "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c": "графика",
}

WHERE_VALUES_RU = (
    "Панель управления → Электропитание → Настройка схемы → Изменить дополнительные параметры. "
    "Для каждого пункта задаются отдельные значения «От сети» и «От батареи»."
)

VISIBILITY_NOTE_RU = (
    "Селектор «Вкл / Выкл / Авто» справа меняет только видимость пункта в том же списке "
    "дополнительных параметров. «Авто» — без явного Attributes в реестре (как у Windows по умолчанию). "
    "Сами числа и режимы «От сети» / «От батареи» здесь не редактируются."
)

WHERE_VALUES_EN = (
    "Control Panel → Power Options → Change plan settings → Change advanced power settings. "
    "Each setting has separate On battery and Plugged in values."
)

VISIBILITY_NOTE_EN = (
    "The On / Off / Auto selector on the right changes only whether the item appears in that "
    "advanced settings list. Auto removes an explicit Attributes value (Windows default). "
    "AC/DC numeric values are not edited here."
)

SUBGROUP_EN = {
    "fea3413e-7e05-4911-9a71-700331f1c294": "general",
    "0012ee47-9041-4b5d-9b77-535fba8b1442": "hard disk",
    "238c9fa8-0aad-41ed-83f4-97be242c8f20": "sleep",
    "2a737441-1930-4402-8d77-b2bebba308a3": "USB",
    "2e601130-5351-4d9d-8e04-252966bad054": "idle resiliency",
    "4f971e89-eebd-4455-a8de-9e59040e7347": "power buttons and lid",
    "501a4d13-42af-4429-9fd1-a8218c268e20": "PCI Express",
    "54533251-82be-4824-96c1-47b60b740d00": "processor",
    "7516b95f-f776-4464-8c53-06167f40cc99": "display",
    "e73a048d-bf27-4f12-9731-8b2076e8891f": "battery",
    "9596fb26-9850-41fd-ac3e-f7c3c00afd4b": "multimedia",
    "de830923-a562-41af-a086-e3a2c6bad2da": "energy saver",
    "8619b916-e004-4dd8-9b66-dae86f806698": "presence aware power",
    "48672f38-7a9a-4bb2-8bf8-3d85be19de4e": "interrupt steering",
    "44f3beca-a7c0-460e-9df2-bb8b99e0cba6": "Intel graphics",
    "5fb4938d-1ee8-4b0f-9a3c-5036b0ab995c": "graphics",
}

TITLE_OVERRIDES = {
    "4faab71a-92e5-4726-b531-224559672d19": "Политика простоя устройств",
    "68afb2d9-ee95-47a8-8f50-4115088073b1": "Режим ожидания при отключении от сети",
    "f15576e8-98b7-4186-b944-eafa664402d9": "Сеть в режиме сна",
    "6738e2c4-e8a5-4a42-b16a-e040e769756e": "Отключение жёсткого диска через",
    "0e796bdb-100d-47d6-a2d5-f7d2daa51f51": "Требовать пароль при пробуждении",
    "245d8541-3943-4422-b025-13a784f679b7": "Тип схемы электропитания",
    "be337238-0d82-4146-a960-4f3749d470c7": "Режим ускорения процессора (Turbo Boost)",
    "93b8b6dc-0698-4d1c-9ee4-0644e900c85d": "Политика планирования потоков (P/E-ядра)",
}


def clean_en(en: str) -> str:
    en = re.sub(r"^[-\d;,]+", "", en or "").strip()
    return en


def title_ru(en: str, guid: str) -> str:
    g = guid.lower()
    if g in MANUAL:
        return MANUAL[g][0]
    if g in TITLE_OVERRIDES:
        return TITLE_OVERRIDES[g]
    en = clean_en(en)
    if not en:
        return f"Параметр ({guid[:8]}…)"
    return en


def _eff_class_tip(en: str) -> str:
    if "efficiency class 1" in en.lower():
        return (
            "Суффикс «Processor Power Efficiency Class 1» — настройка для экономных (E-) ядер "
            "в гибридных CPU Intel; одноимённый пункт без суффикса обычно для P-ядер."
        )
    return ""


def explain_detail(en: str, subgroup: str) -> tuple[str, str, str]:
    """Назначение, поведение, практика — без пустых блоков в compose."""
    en_l = en.lower()
    sg = SUBGROUP_RU.get(subgroup.lower(), "электропитание")
    extra = _eff_class_tip(en)

    def pack(purpose: str, behavior: str, tips: str = "") -> tuple[str, str, str]:
        if extra and extra not in tips:
            tips = (tips + " " + extra).strip() if tips else extra
        return purpose, behavior, tips

    if "core parking" in en_l:
        return pack(
            "Управляет «парковкой» ядер: сколько логических процессоров Windows держит активными при низкой нагрузке.",
            "Больше активных ядер — быстрее отклик на всплески, выше простой расход и нагрев. "
            "Агрессивная парковка экономит батарею, но может давать микрофризы при старте нагрузки.",
            "При «плавающем» FPS или задержках ввода попробуйте поднять минимум активных ядер от батареи. "
            "Для офиса и серфинга заводские значения обычно достаточны.",
        )
    if "boost mode" in en_l or "boost policy" in en_l:
        return pack(
            "Задаёт, насколько агрессивно процессор использует Turbo Boost (кратковременное повышение частоты).",
            "«Enabled» / агрессивные политики — выше пики производительности и нагрев. "
            "«Disabled» или «Efficient» — стабильнее температура и тише вентилятор, ниже пики.",
            "На ноутбуке от батареи часто разумно ослабить boost; от сети — оставить включённым для игр и рендера.",
        )
    if "processor performance" in en_l and ("increase" in en_l or "decrease" in en_l):
        return pack(
            "Тонкая настройка того, как быстро менеджер питания CPU поднимает или снижает частоту при изменении нагрузки.",
            "Короткие интервалы и низкие пороги — частота реагирует острее; длинные — плавнее, но с задержкой отклика.",
            "Меняйте только если понимаете цель (латентность vs стабильность). Иначе оставьте значения схемы «Сбалансированная».",
        )
    if "processor idle" in en_l or ("idle" in en_l and subgroup.lower().startswith("54533251")):
        return pack(
            "Определяет глубину и скорость перехода ядер в состояния простоя (C-states).",
            "Глубокий простой сильнее экономит ватты, но удлиняет пробуждение ядра на доли миллисекунды.",
            "При аудио/DAW или низкой задержке иногда отключают часть idle; для автономности — заводские настройки.",
        )
    if "frequency" in en_l or "throttl" in en_l:
        return pack(
            "Ограничивает минимальную или максимальную частоту процессора (часто в процентах от номинала).",
            "Потолок ниже 100% — меньше нагрев и шум, но ниже производительность. "
            "Пол — не даёт CPU опускаться слишком низко при кратких нагрузках.",
            "Полезно на тонких ноутбуках с перегревом: слегка снизить максимум от батареи, не трогая от сети.",
        )
    if "cooling policy" in en_l:
        return pack(
            "Выбирает приоритет: держать частоту (активное охлаждение) или снижать её, чтобы уменьшить нагрев (пассивное).",
            "«Active» — вентилятор крутится раньше, CPU дольше держит turbo. "
            "«Passive» — сначала снижается частота, тише, но возможен throttling под длинной нагрузкой.",
            "Для игр от сети — Active; для тихой работы от батареи — Passive или баланс схемы.",
        )
    if "latency sensitivity" in en_l:
        return pack(
            "Подсказки для процессов, чувствительных к задержке: сколько ядер и какой минимум производительности резервировать.",
            "Больше «зарезервированных» ресурсов — стабильнее input lag и таймеры, выше фоновый расход.",
            "Имеет смысл при проблемах с мышью/аудио; в остальных случаях не трогать.",
        )
    if "heterogeneous" in en_l and "scheduling" in en_l:
        return pack(
            "Для CPU с P- и E-ядрами: куда планировщик чаще отправляет потоки.",
            "Performant — больше работы на P-ядрах (скорость, нагрев). Efficient — на E-ядрах (батарея, тишина). "
            "Automatic — решение Windows.",
            "На однородных CPU (большинство Ryzen, старые Intel) эффект минимален.",
        )
    if "timeout" in en_l or "таймаут" in en_l:
        return pack(
            f"Таймер бездействия в разделе «{sg}»: через сколько срабатывает связанное действие (сон, отключение, переход состояния).",
            "Короткий интервал — быстрее экономия, чаще «просыпания» устройств. "
            "0 или «Никогда» — автоматика отключена, ресурс остаётся включённым.",
            "От батареи обычно короче, от сети длиннее. Для дисков и NVMe — баланс между энергией и задержкой первого чтения.",
        )
    if "policy" in en_l or "polic" in en_l:
        return pack(
            f"Политика (режим) в группе «{sg}»: выбор между экономией, производительностью или автоматическим балансом.",
            "Агрессивная экономия снижает потребление и может добавить задержки; режим производительности — наоборот.",
            "Сначала сравните готовые схемы Windows; точечная политика — когда одного пункта в схеме не хватает.",
        )
    if "threshold" in en_l:
        return pack(
            "Числовой порог: при какой нагрузке, задержке или счётчике срабатывает следующий шаг (частота, парковка, простой).",
            "Ниже порог — раньше реакция, возможны лишние переключения. Выше — стабильнее, но медленнее отклик.",
            "Меняйте небольшими шагами и проверяйте на типичной задаче (игра, браузер, рендер).",
        )
    if "minimum" in en_l or "maximum" in en_l or "bound" in en_l:
        return pack(
            f"Жёсткий или мягкий предел (минимум/максимум) для параметра в «{sg}».",
            "Сужает диапазон, в котором система может автоматически подстраиваться под нагрузку.",
            "Используйте, чтобы ограничить перегрев или разряд; не ставьте минимум и максимум слишком близко — CPU потеряет гибкость.",
        )
    if "sleep" in en_l or "standby" in en_l or "hibern" in en_l or "fasts4" in en_l:
        return pack(
            "Управление сном, гибернацией и смежными режимами ожидания.",
            "Сон — быстрое пробуждение, питание RAM; гибернация — дольше уходит в сон, почти не тратит батарею на хранение. "
            "Таймауты определяют, когда переход происходит без вашего участия.",
            "Если ПК «засыпает сам» — проверьте таймауты от батареи и фоновые задачи. Для скачиваний — увеличить или отключить для сети.",
        )
    if "away mode" in en_l or "unattended" in en_l or "remote opens" in en_l:
        return pack(
            "Разрешает или запрещает уход в сон при медиа, удалённом доступе или «away mode».",
            "Запрет сна нужен для фонового воспроизведения и некоторых серверных сценариев; иначе растёт расход энергии.",
            "Для домашнего ПК обычно достаточно стандартных политик; включайте исключения только под конкретную программу.",
        )
    if "disk" in en_l or "hard" in en_l or "nvme" in en_l or "ahci" in en_l:
        return pack(
            "Питание накопителя: AHCI Link Power Management, таймауты простоя NVMe, отключение шпинделя (HDD).",
            "Агрессивная экономия снижает ватты в idle; первое обращение после простоя может занять чуть больше времени.",
            "На SSD/NVMe не отключайте всё «в ноль» без причины — разумный idle timeout часто лучший компромисс.",
        )
    if "usb" in en_l:
        return pack(
            "Selective suspend USB: отключение портов и хабов при простое.",
            "Экономит батарею на ноутбуке; флешки, мыши и внешние диски могут «отваливаться» до переподключения.",
            "При нестабильных USB-устройствах отключите selective suspend для проблемного порта или всей схемы от сети.",
        )
    if "pci" in en_l or "express" in en_l or "link state" in en_l or "link power" in en_l:
        return pack(
            "Состояние линка PCI Express (GPU, NVMe, контроллеры) в простое.",
            "ASPM и пониженные состояния линка уменьшают потребление; wake latency при пиковой нагрузке может чуть вырасти.",
            "На настольном ПК от сети часто оставляют максимальную производительность линка; на ноутбуке — умеренную экономию.",
        )
    if "button" in en_l or "lid" in en_l or ("power" in en_l and "action" in en_l):
        return pack(
            "Действие при нажатии кнопки питания/сна, закрытии крышки или схожих аппаратных событиях.",
            "Сон и гибернация сохраняют работу; выключение завершает сеанс; «Ничего не делать» — только для крышки, если нужен работа с закрытой крышкой от док-станции.",
            "На ноутбуке «Закрытие крышки → сон» — типичный выбор; для док-станции часто «ничего» от сети.",
        )
    if "battery" in en_l or "charge" in en_l or "reserve" in en_l or "critical" in en_l:
        return pack(
            "Пороги заряда, резерв, критический уровень и связанные уведомления.",
            "Раннее предупреждение даёт время сохранить работу; низкий резерв может принудительно усыплять или гибернировать.",
            "Критический уровень не опускайте слишком низко — риск потери данных при внезапном отключении.",
        )
    if "display" in en_l or "brightness" in en_l or "video" in en_l or "экран" in en_l:
        return pack(
            "Экран и видео: яркость, таймаут выключения, адаптивная яркость, поведение при воспроизведении.",
            "Таймаут экрана сильнее всего влияет на автономность; яркость — второй по значимости фактор.",
            "«Выключать дисплей через …» от батареи обычно 3–5 мин; от сети — дольше или никогда.",
        )
    if "interrupt" in en_l or "steering" in en_l:
        return pack(
            "Interrupt steering: на какие ядра направлять аппаратные прерывания устройств.",
            "Распределение влияет на задержку и равномерность нагрузки; неправильный режим может добавить jitter.",
            "Трогать имеет смысл при диагностике DPC latency; для обычного использования — значения по умолчанию.",
        )
    if "graphics" in en_l or "intel" in en_l and "power" in en_l:
        return pack(
            "План питания встроенной графики Intel (частоты, turbo, простой).",
            "Баланс между FPS в лёгких играх и временем работы от батареи при использовании iGPU.",
            "При дискретной NVIDIA/AMD основная нагрузка часто на другом GPU — эффект зависит от переключения графики.",
        )
    if "resilien" in en_l or "coalesc" in en_l or "deep sleep" in en_l:
        return pack(
            "Idle Resiliency: глубина простоя CPU, объединение I/O и таймауты «execution required».",
            "Углублённый простой и coalescing снижают ватты в фоне; возможны редкие задержки отклика на I/O.",
            "Параметры для тонкой настройки и отладки; без явной проблемы оставьте схему по умолчанию.",
        )
    if "lock console" in en_l or "password" in en_l or "wake" in en_l:
        return pack(
            "Поведение при пробуждении: блокировка экрана, пароль, разрешение wake-таймеров и устройств.",
            "Требование пароля повышает безопасность; лишние wake-источники будят ПК ночью и тратят батарею.",
            "На ноутбуке отключите пробуждение от сетевой карты, если не нужен Wake-on-LAN.",
        )
    if "personality" in en_l or "scheme" in en_l:
        return pack(
            "Служебная или обобщающая настройка схемы (тип профиля электропитания).",
            "Влияет на то, как Windows классифицирует схему для некоторых подсистем; редко меняется вручную.",
            "Обычно достаточно выбрать готовую схему в панели электропитания, а не править этот пункт.",
        )
    return pack(
        f"Параметр раздела «{sg}» в схеме электропитания Windows.",
        "Участвует в балансе между производительностью, нагревом и временем работы от батареи; точный эффект зависит от железа и драйверов.",
        f"Ориентируйтесь на имя в Windows: «{en}». При сомнении — оставьте значение схемы по умолчанию.",
    )


def section(title: str, text: str) -> str | None:
    t = (text or "").strip()
    if not t:
        return None
    return f"{title}\n\n{t}"


def compose_body_ru(title: str, en: str, sg_name: str, purpose: str, behavior: str, tips: str) -> str:
    intro = f"«{title}» · раздел «{sg_name}»."
    if en and en.strip() and title != en:
        intro += f"\nИмя в Windows: «{en}»."

    blocks: list[str] = [intro]
    for block in (
        section("Назначение", purpose),
        section("Поведение", behavior),
        section("На практике", tips),
        section("Где задать «От сети» / «От батареи»", WHERE_VALUES_RU),
        section("Видимость в этой программе", VISIBILITY_NOTE_RU),
    ):
        if block:
            blocks.append(block)
    return "\n\n".join(blocks)


def compose_body_en(title: str, en: str, sg_name: str, purpose: str, behavior: str, tips: str) -> str:
    intro = f"“{title}” · section “{sg_name}”."
    if en and en.strip() and title != en:
        intro += f"\nWindows name: “{en}”."

    blocks: list[str] = [intro]
    for block in (
        section("Purpose", purpose),
        section("Behavior", behavior),
        section("In practice", tips),
        section("Where to set On battery / Plugged in", WHERE_VALUES_EN),
        section("Visibility in this program", VISIBILITY_NOTE_EN),
    ):
        if block:
            blocks.append(block)
    return "\n\n".join(blocks)


def title_en(en: str, guid: str) -> str:
    g = guid.lower()
    en = clean_en(en)
    if g in MANUAL:
        # Prefer Windows EN name when available; else Russian title as last resort is avoided.
        return en or MANUAL[g][0]
    if not en:
        return f"Setting ({guid[:8]}…)"
    return en


def _eff_class_tip_en(en: str) -> str:
    if "efficiency class 1" in en.lower():
        return (
            "The “Processor Power Efficiency Class 1” suffix is for efficient (E-) cores on hybrid "
            "Intel CPUs; the matching item without the suffix is usually for P-cores."
        )
    return ""


def explain_detail_en(en: str, subgroup: str) -> tuple[str, str, str]:
    en_l = en.lower()
    sg = SUBGROUP_EN.get(subgroup.lower(), "power")
    extra = _eff_class_tip_en(en)

    def pack(purpose: str, behavior: str, tips: str = "") -> tuple[str, str, str]:
        if extra and extra not in tips:
            tips = (tips + " " + extra).strip() if tips else extra
        return purpose, behavior, tips

    if "core parking" in en_l:
        return pack(
            "Controls core parking: how many logical processors Windows keeps active under light load.",
            "More active cores improve burst responsiveness but raise idle power and heat. "
            "Aggressive parking saves battery but may add micro-stutters when load starts.",
            "If FPS hitches or input feels delayed, try raising the minimum unparked cores on battery.",
        )
    if "boost mode" in en_l or "boost policy" in en_l:
        return pack(
            "How aggressively the CPU uses Turbo Boost (short frequency increases).",
            "Enabled / aggressive policies raise peak performance and heat. "
            "Disabled or Efficient keeps temperatures steadier with lower peaks.",
            "On battery, easing boost is often reasonable; on AC, leave it on for games and renders.",
        )
    if "timeout" in en_l:
        return pack(
            f"Idle timer in the “{sg}” section: how long before the related action (sleep, power-down, state change).",
            "Shorter intervals save power sooner but wake devices more often. "
            "0 or Never disables the automation.",
            "On battery usually shorter; on AC longer. Balance energy vs first-access latency for disks/NVMe.",
        )
    if "policy" in en_l or "polic" in en_l:
        return pack(
            f"A policy (mode) in “{sg}”: choose economy, performance, or automatic balance.",
            "Aggressive economy lowers power and may add latency; performance mode does the opposite.",
            "Start from a built-in Windows plan; tune a single policy only when the plan is not enough.",
        )
    if "sleep" in en_l or "standby" in en_l or "hibern" in en_l or "fasts4" in en_l:
        return pack(
            "Controls sleep, hibernate, and related standby modes.",
            "Sleep wakes quickly and keeps RAM powered; hibernate uses almost no battery while off. "
            "Timeouts decide when transitions happen unattended.",
            "If the PC sleeps unexpectedly, check battery timeouts and background tasks.",
        )
    if "disk" in en_l or "hard" in en_l or "nvme" in en_l or "ahci" in en_l:
        return pack(
            "Storage power: AHCI Link Power Management, NVMe idle timeouts, HDD spin-down.",
            "Aggressive savings cut idle watts; the first access after idle may take slightly longer.",
            "On SSD/NVMe avoid extreme “always off” settings without a reason — a moderate idle timeout is usually best.",
        )
    if "usb" in en_l:
        return pack(
            "USB selective suspend: powering down ports/hubs when idle.",
            "Saves laptop battery; flash drives, mice, or external disks may disconnect until replugged.",
            "If a USB device is unstable, disable selective suspend for that plan.",
        )
    if "button" in en_l or "lid" in en_l or ("power" in en_l and "action" in en_l):
        return pack(
            "Action for the power/sleep button, lid close/open, or similar hardware events.",
            "Sleep/hibernate keep your session; Shut down ends it; Do nothing is useful for docked lids on AC.",
            "Laptops commonly use Lid close → Sleep; docks often use Do nothing on AC.",
        )
    if "battery" in en_l or "charge" in en_l or "reserve" in en_l or "critical" in en_l:
        return pack(
            "Charge thresholds, reserve level, critical battery, and related notifications.",
            "Earlier warnings give time to save work; a low reserve may force sleep or hibernate.",
            "Do not set the critical level too low — risk of data loss on sudden power loss.",
        )
    if "display" in en_l or "brightness" in en_l or "video" in en_l:
        return pack(
            "Display and video: brightness, turn-off timeout, adaptive brightness, playback behavior.",
            "Display timeout has the largest impact on battery life; brightness is second.",
            "On battery, 3–5 minutes to turn off the display is common; on AC, longer or never.",
        )
    if "processor" in en_l or "frequency" in en_l or "throttl" in en_l or "idle" in en_l:
        return pack(
            f"Processor power management in “{sg}”: frequency, idle states, or related limits.",
            "Tighter limits reduce heat and noise but can cut performance; looser limits do the opposite.",
            "Change gradually and verify on a typical workload.",
        )
    return pack(
        f"A power setting in the “{sg}” section of the Windows power plan.",
        "It participates in the balance of performance, heat, and battery life; exact effect depends on hardware and drivers.",
        f"Use the Windows name “{en}” as a guide. When unsure, keep the plan defaults.",
    )


def body_ru(en: str, guid: str, subgroup: str) -> str:
    g = guid.lower()
    if g in MANUAL:
        return MANUAL[g][1]

    en = clean_en(en)
    title = title_ru(en, guid)
    sg_name = SUBGROUP_RU.get(subgroup.lower(), "другие параметры")

    if not en:
        return compose_body_ru(
            f"Параметр ({g[:8]}…)",
            "",
            sg_name,
            "Служебный или OEM-параметр без отображаемого имени в экспорте.",
            "Может относиться к драйверу или скрытой возможности конкретного производителя.",
            "Меняйте только если знаете документацию производителя.",
        )

    purpose, behavior, tips = explain_detail(en, subgroup)
    return compose_body_ru(title, en, sg_name, purpose, behavior, tips)


def body_en(en: str, guid: str, subgroup: str) -> str:
    g = guid.lower()
    en = clean_en(en)
    title = title_en(en, guid)
    sg_name = SUBGROUP_EN.get(subgroup.lower(), "other settings")

    if g == "93b8b6dc-0698-4d1c-9ee4-0644e900c85d":
        return compose_body_en(
            "Heterogeneous thread scheduling policy",
            "Heterogeneous thread scheduling policy",
            "processor",
            "Hints the Windows scheduler whether to prefer performant (P) or efficient (E) cores.",
            "Prefer performant — more work on P-cores (responsiveness, heat, battery use). "
            "Prefer efficient — more work on E-cores (cooler, quieter, longer battery). "
            "Automatic — Windows chooses the balance.",
            "On CPUs without distinct core types (most Ryzen, older Intel) the effect is minimal.",
        )

    if not en:
        return compose_body_en(
            f"Setting ({g[:8]}…)",
            "",
            sg_name,
            "A service or OEM setting without a display name in the export.",
            "It may belong to a driver or a vendor-specific capability.",
            "Change it only if you have vendor documentation.",
        )

    purpose, behavior, tips = explain_detail_en(en, subgroup)
    return compose_body_en(title, en, sg_name, purpose, behavior, tips)


def esc(s: str) -> str:
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\r", "")
        .replace("\n", "\\n")
    )


def main():
    seen = {}
    rows = []
    with CSV.open(encoding="utf-8-sig", newline="") as f:
        for row in csv.DictReader(f):
            g = row["Setting"].lower()
            if g in seen:
                continue
            seen[g] = True
            rows.append(row)

    lines = [
        "// Auto-generated by scripts/generate_catalog.py",
        "",
        "use crate::i18n::Lang;",
        "",
        "pub struct CatalogEntry {",
        "    pub guid: &'static str,",
        "    pub title_ru: &'static str,",
        "    pub title_en: &'static str,",
        "    pub body_ru: &'static str,",
        "    pub body_en: &'static str,",
        "}",
        "",
        "impl CatalogEntry {",
        "    pub fn title(&self, lang: Lang) -> &'static str {",
        "        match lang {",
        "            Lang::Ru => self.title_ru,",
        "            Lang::En => self.title_en,",
        "        }",
        "    }",
        "",
        "    pub fn body(&self, lang: Lang) -> &'static str {",
        "        match lang {",
        "            Lang::Ru => self.body_ru,",
        "            Lang::En => self.body_en,",
        "        }",
        "    }",
        "}",
        "",
        "pub static ENTRIES: &[CatalogEntry] = &[",
    ]
    for row in rows:
        g = row["Setting"].lower()
        sg = row.get("Subgroup", "")
        en_name = row["EN"]
        tr = esc(title_ru(en_name, g))
        te = esc(title_en(en_name, g))
        br = esc(body_ru(en_name, g, sg))
        be = esc(body_en(en_name, g, sg))
        lines.extend(
            [
                "    CatalogEntry {",
                f'        guid: "{g}",',
                f'        title_ru: "{tr}",',
                f'        title_en: "{te}",',
                f'        body_ru: "{br}",',
                f'        body_en: "{be}",',
                "    },",
            ]
        )
    lines.extend(
        [
            "];",
            "",
            "pub fn lookup(setting_guid: &str) -> Option<&'static CatalogEntry> {",
            "    let g = setting_guid.to_lowercase();",
            "    ENTRIES.iter().find(|e| e.guid == g)",
            "}",
            "",
        ]
    )
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {len(rows)} bilingual entries to {OUT}")


if __name__ == "__main__":
    main()
