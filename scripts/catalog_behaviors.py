# -*- coding: utf-8 -*-
"""Уникальные тексты «Поведение» (и связанные блоки) по имени параметра Windows."""

from __future__ import annotations

import re
from typing import Callable

Detail = tuple[str, str, str]  # purpose, behavior, tips


def _norm(en: str) -> str:
    return re.sub(r"^[-\d;,]+", "", en or "").strip().lower()


def _eff_suffix(en_l: str) -> str:
    if "efficiency class 1" in en_l:
        return (
            " Относится к экономным (E-) ядрам гибридного CPU; одноимённый пункт без этого суффикса "
            "обычно настраивает P-ядра."
        )
    if "efficiency class 0" in en_l:
        return " Задаёт нижний предел для P-ядер (Class 0), когда часть E-ядер разпаркована."
    return ""


def _eff_suffix_en(en_l: str) -> str:
    if "efficiency class 1" in en_l:
        return (
            " Applies to efficient (E-) cores on hybrid CPUs; the same setting without this suffix "
            "usually targets P-cores."
        )
    if "efficiency class 0" in en_l:
        return " Sets a floor for P-cores (Class 0) while some E-cores are unparked."
    return ""


def _pack_ru(purpose: str, behavior: str, tips: str, en_l: str, en_c: str = "") -> Detail:
    if en_c and en_c.lower() not in behavior.lower():
        behavior = f"«{en_c}»: {behavior}"
    suf = _eff_suffix(en_l)
    if suf and suf.strip() not in behavior:
        behavior = (behavior.rstrip() + suf).strip()
    return purpose, behavior, tips


def _pack_en(purpose: str, behavior: str, tips: str, en_l: str) -> Detail:
    suf = _eff_suffix_en(en_l)
    if suf and suf.strip() not in behavior:
        behavior = (behavior.rstrip() + suf).strip()
    return purpose, behavior, tips


# --- Точные совпадения по имени (приоритет выше эвристик) ---

_EXACT_RU: dict[str, Detail] = {}
_EXACT_EN: dict[str, Detail] = {}


def _reg(name: str, ru: Detail, en: Detail) -> None:
    key = name.strip().lower()
    _EXACT_RU[key] = ru
    _EXACT_EN[key] = en


_reg(
    "AHCI Link Power Management - HIPM/DIPM",
    (
        "Режим управления питанием SATA-линка через Host Initiated (HIPM) и Device Initiated (DIPM).",
        "HIPM — хост инициирует переходы линка в экономичные состояния; DIPM — накопитель сам просит сон линка. "
        "«Active» держит линк готовым чаще (быстрее первый доступ после простоя). "
        "«Lowest» / агрессивные режимы дольше держат линк в сниженном питании — меньше ватты, возможна задержка пробуждения диска.",
        "На SSD с частым случайным I/O не выкручивайте экономию «в ноль» от сети; от батареи умеренный HIPM/DIPM обычно оправдан.",
    ),
    (
        "SATA link power mode using Host Initiated (HIPM) and Device Initiated (DIPM) transitions.",
        "HIPM lets the host drive link low-power states; DIPM lets the drive request them. "
        "Active keeps the link ready more often (faster first access after idle). "
        "Lowest / aggressive modes stay in reduced power longer — less idle power, more wake latency.",
        "On SSDs with frequent random I/O, avoid extreme savings on AC; moderate HIPM/DIPM on battery is usually reasonable.",
    ),
)

_reg(
    "AHCI Link Power Management - Adaptive",
    (
        "Адаптивный AHCI: Windows подстраивает агрессивность HIPM/DIPM под недавнюю активность диска.",
        "После серии обращений линк дольше остаётся «бодрым»; в длительном простое постепенно углубляет экономию. "
        "Это компромисс между фиксированным «всегда экономить» и «никогда не экономить».",
        "Если заметны редкие «подвисания» первого чтения после паузы — смягчите режим от батареи, не трогая сеть.",
    ),
    (
        "Adaptive AHCI: Windows adjusts HIPM/DIPM aggressiveness based on recent disk activity.",
        "After bursts of I/O the link stays ready longer; after long idle it deepens power savings. "
        "This sits between fixed always-save and never-save policies.",
        "If the first read after idle occasionally stalls, ease the setting on battery and leave AC alone.",
    ),
)

_reg(
    "Harddisk power down timeout",
    (
        "Через сколько секунд простоя HDD останавливается шпиндель (отключение механики).",
        "По истечении таймера диск раскручивается вниз — падает потребление и шум. "
        "Следующее обращение ждёт раскрутку (секунды), что заметно на старых HDD.",
        "На чисто SSD-системах пункт часто не влияет; на гибридах (SSHD) имеет смысл не ставить слишком короткий таймаут от сети.",
    ),
    (
        "Seconds of disk idle before the HDD motor stops (spin-down).",
        "When the timer fires, the platters stop — less power and noise. "
        "The next access waits for spin-up (seconds), noticeable on classic HDDs.",
        "Often irrelevant on SSD-only systems; on SSHD hybrids avoid an overly short timeout on AC.",
    ),
)

_reg(
    "Hard disk burst ignore time",
    (
        "Окно после всплеска активности диска, в которое таймер простоя не перезапускается агрессивно.",
        "Короткие серии мелких запросов (индексация, фоновые задачи) не дают диску мгновенно уйти в power-down. "
        "Большее значение — реже лишние циклы сон/пробуждение; меньшее — быстрее экономия после паузы.",
        "При «дёргающемся» HDD от частых spin-up/down увеличьте ignore time от батареи.",
    ),
    (
        "Grace period after disk activity bursts before idle timers aggressively restart.",
        "Short bursts of small I/O (indexing, background tasks) won't immediately trigger spin-down. "
        "Higher values reduce sleep/wake cycles; lower values save power sooner after idle.",
        "If an HDD keeps spinning up/down, raise ignore time on battery.",
    ),
)

_reg(
    "Maximum Power Level",
    (
        "Верхний предел мощности/производительности накопителя в рамках политики питания диска.",
        "Ограничивает, насколько «агрессивно» диск может держать высокие режимы при нагрузке. "
        "Ниже потолок — меньше пиковый расход и нагрев контроллера; выше — ближе к максимальной отдаче при очереди I/O.",
        "Меняйте только при известных проблемах перегрева NVMe/HDD в тонком корпусе.",
    ),
    (
        "Upper cap on storage performance/power within the disk power policy.",
        "Limits how aggressively the drive may stay in high-performance states under load. "
        "A lower cap cuts peak power and controller heat; a higher cap favors throughput under queue pressure.",
        "Tune only when you know the drive thermally throttles in a thin chassis.",
    ),
)

_reg(
    "Primary NVMe Idle Timeout",
    (
        "Сколько простаивает основной NVMe, прежде чем контроллер уйдёт в более глубокое энергосбережение.",
        "Короткий таймаут быстрее снижает ватты в idle; wake из глубокого состояния может добавить микросекунды–миллисекунды к первому I/O. "
        "«Primary» — первый/системный NVMe в политике Microsoft для дисков.",
        "Для игрового NVMe от сети часто оставляют длиннее; от батареи — короче.",
    ),
    (
        "How long the primary NVMe may idle before the controller enters a deeper save state.",
        "Shorter timeouts cut idle watts faster; waking from deep states may add microseconds–milliseconds to the first I/O. "
        "Primary is the main/system NVMe in Microsoft's disk policy.",
        "On AC, gaming NVMe often uses a longer timeout; on battery, shorter.",
    ),
)

_reg(
    "Secondary NVMe Idle Timeout",
    (
        "Таймаут простоя для дополнительного NVMe (не системного диска).",
        "Логика как у Primary, но применяется ко второму накопителю — удобно агрессивнее экономить на «данном» диске, "
        "не трогая системный том с частым swap/файлом подкачки.",
        "Если второй NVMe — архив/медиа, короткий idle от батареи обычно безопасен.",
    ),
    (
        "Idle timeout for a secondary (non-system) NVMe.",
        "Same idea as Primary, but for the extra drive — you can save more aggressively on data volumes "
        "without affecting the OS disk with frequent paging/temp I/O.",
        "If the second NVMe is bulk storage, a short idle timeout on battery is usually fine.",
    ),
)

_reg(
    "Primary NVMe Power State Transition Latency Tolerance",
    (
        "Насколько система терпит задержку при выходе primary NVMe из глубокого power state.",
        "Больше tolerance — разрешены более глубокие состояния простоя (ниже ватты). "
        "Меньше — контроллер держится ближе к готовности, быстрее отклик, выше фоновый расход.",
        "При stutter в играх с диска на NVMe попробуйте снизить tolerance от сети.",
    ),
    (
        "How much latency is acceptable when the primary NVMe leaves a deep power state.",
        "Higher tolerance allows deeper idle states (lower watts). "
        "Lower tolerance keeps the drive nearer ready — snappier, more idle power.",
        "If you see game hitches tied to storage, try lowering tolerance on AC.",
    ),
)

_reg(
    "Secondary NVMe Power State Transition Latency Tolerance",
    (
        "Допустимая задержка пробуждения secondary NVMe из энергосберегающих состояний.",
        "Отдельно от primary: можно разрешить глубокий сон второму диску, не увеличивая latency системного тома.",
        "Для второго диска с редким доступом высокий tolerance от батареи — нормальная экономия.",
    ),
    (
        "Acceptable wake latency for the secondary NVMe leaving low-power states.",
        "Independent from primary: deep sleep on the data drive without hurting OS volume latency.",
        "For rarely accessed secondary drives, high tolerance on battery is reasonable.",
    ),
)

_reg(
    "NVMe NOPPME",
    (
        "Управление NOPPME (Non-Operational Power State Permissive Mode Enable) для NVMe.",
        "Влияет на то, может ли накопитель входить в неоперационные power states при простое шины/системы. "
        "Отключение или жёсткие режимы могут удерживать устройство ближе к активному состоянию.",
        "Тонкая настройка; без проблем с wake latency лучше не менять.",
    ),
    (
        "Controls NVMe NOPPME (Non-Operational Power State Permissive Mode Enable).",
        "Affects whether the drive may enter non-operational power states during platform idle. "
        "Disabling or strict modes can keep the device nearer active states.",
        "Advanced tuning — leave default unless you diagnose NVMe wake issues.",
    ),
)

_reg(
    "Heterogeneous thread scheduling policy",
    (
        "Рекомендация планировщику Windows для гибридных CPU (P- и E-ядра).",
        "Prefer performant смещает потоки на P-ядра — выше однопоточная отдача и нагрев. "
        "Prefer efficient — на E-ядра: тише и дольше батарея. Automatic — баланс силами Windows.",
        "На однородных CPU эффект минимален; меняйте отдельно для сети и батареи.",
    ),
    (
        "Scheduler hint for hybrid CPUs with P- and E-cores.",
        "Prefer performant shifts threads toward P-cores — better single-thread peaks and heat. "
        "Prefer efficient favors E-cores — cooler, longer battery. Automatic lets Windows balance.",
        "Minimal effect on homogeneous CPUs; tune AC vs battery separately.",
    ),
)

_reg(
    "Heterogeneous short running thread scheduling policy",
    (
        "Куда отправлять короткоживущие потоки на гибридном CPU.",
        "Короткие задачи часто не успевают «окупить» миграцию на P-ядро; политика решает, "
        "оставлять ли их на E-ядрах или всё же поднимать на P для минимальной задержки.",
        "При «микролагах» UI попробуйте performant от сети; для автономности — efficient от батареи.",
    ),
    (
        "Where short-lived threads run on hybrid CPUs.",
        "Brief work may not amortize migration to P-cores; this policy chooses E-core residency vs a quick P-core boost for latency.",
        "For UI micro-stutters try performant on AC; for battery life try efficient on battery.",
    ),
)

_reg(
    "Heterogeneous policy in effect.",
    (
        "Служебный индикатор: какая гибридная политика планирования сейчас активна в схеме.",
        "Не задаёт поведение напрямую — отражает результат выбранных политик и возможностей CPU. "
        "Смена «вручную» обычно не требуется.",
        "Ориентируйтесь на явные пункты heterogeneous scheduling policy, а не на этот индикатор.",
    ),
    (
        "Service indicator of which heterogeneous scheduling policy is in effect.",
        "Does not directly set behavior — reflects chosen policies and CPU capabilities. "
        "Manual changes are rarely needed.",
        "Tune explicit heterogeneous scheduling policies instead of this indicator.",
    ),
)

# Sleep / power buttons / battery / display — exact entries for settings that shared generic templates

_sleep_exact: list[tuple[str, Detail, Detail]] = [
    (
        "System standby timeout",
        (
            "Через сколько минут бездействия ПК переходит в сон (S3/modern standby).",
            "Сон сохраняет сеанс в RAM и быстро просыпается; таймер срабатывает без подтверждения пользователя. "
            "0 отключает автоматический переход — ПК остаётся включённым до явного действия.",
            "От батареи обычно 5–15 мин; от сети — дольше или «Никогда» для рабочей станции.",
        ),
        (
            "Minutes of inactivity before the PC enters sleep (S3/modern standby).",
            "Sleep keeps the session in RAM and wakes quickly; the timer fires without asking. "
            "0 disables automatic sleep — the PC stays on until you act.",
            "Battery often 5–15 minutes; AC longer or Never for desktops.",
        ),
    ),
    (
        "System hibernation timeout",
        (
            "Через сколько минут бездействия выполняется переход в гибернацию.",
            "Гибернация сбрасывает RAM на диск и почти не тратит батарею; пробуждение медленнее сна. "
            "Часто используется как «глубокий» таймаут после сна или на старых ноутбуках.",
            "Если гибернация не нужна — 0; иначе ставьте больше, чем standby timeout.",
        ),
        (
            "Minutes of inactivity before hibernate.",
            "Hibernate writes RAM to disk and uses little battery while off; wake is slower than sleep. "
            "Often a deeper timeout after sleep or on older laptops.",
            "Set 0 if you don't use hibernate; otherwise longer than standby timeout.",
        ),
    ),
    (
        "System unattended sleep timeout",
        (
            "Таймаут сна, когда пользователь считается «отошедшим», но сеанс может оставаться активным.",
            "Отличается от обычного standby: учитывает сценарии без ввода (медиа, презентации, удалённая работа). "
            "Короче — быстрее уход в сон при отсутствии пользователя.",
            "Настройте в паре с «Allow sleep with remote opens» и медиа-политиками.",
        ),
        (
            "Sleep timeout when the user is considered away but a session may still be active.",
            "Differs from basic standby — covers no-input scenarios (media, presentations, remote use). "
            "Shorter values sleep sooner when you're away.",
            "Tune alongside Allow sleep with remote opens and media policies.",
        ),
    ),
    (
        "System FastS4 Support",
        (
            "Поддержка ускоренного перехода в гибернацию (Fast S4 / hybrid shutdown patterns).",
            "Влияет на то, как Windows комбинирует сон и гибернацию при определённых схемах и аппаратной поддержке. "
            "Может ускорить «выключение с сохранением состояния» на совместимых системах.",
            "Меняйте только при понимании, что гибернация включена и файл hiberfil.sys присутствует.",
        ),
        (
            "Fast S4 / accelerated hibernate-related shutdown behavior where supported.",
            "Changes how Windows combines sleep and hibernate on compatible hardware. "
            "Can speed state-preserving shutdown paths.",
            "Change only if hibernate is enabled and hiberfil.sys exists.",
        ),
    ),
    (
        "Allow Standby States",
        (
            "Разрешает или запрещает переходы в состояния standby (S1–S3/modern standby).",
            "«Allow» — таймауты и действия кнопок могут усыплять систему. "
            "«Prohibit» блокирует standby — полезно для серверов/стендов, но повышает расход от батареи.",
            "Если ПК никогда не должен засыпать — Prohibit от сети; для ноутбука обычно Allow.",
        ),
        (
            "Permits or blocks transitions into standby states (S1–S3/modern standby).",
            "Allow — timers and buttons may sleep the system. "
            "Prohibit blocks standby — useful for kiosks/servers, bad for battery life.",
            "Prohibit on AC only for always-on setups; laptops usually Allow.",
        ),
    ),
    (
        "Allow Away Mode Policy",
        (
            "Разрешает режим Away Mode — система выглядит «бодрствующей» для медиа/сетевых задач при экономии.",
            "Away Mode может не гасить дисплей и не уходить в классический сон, пока идёт фоновое воспроизведение. "
            "Запрет упрощает предсказуемый сон по таймерам.",
            "Для HTPC/медиа-центра иногда включают; для ноутбука чаще запрещают от батареи.",
        ),
        (
            "Allows Away Mode — the system stays semantically awake for media/network while saving power.",
            "Away Mode may avoid classic sleep/display off during background playback. "
            "Disabling makes timer-driven sleep more predictable.",
            "Sometimes enabled on HTPCs; laptops often disable on battery.",
        ),
    ),
    (
        "Allow System Required Policy",
        (
            "Может ли система оставаться «System Required» (не уходить в сон) при активных power requests.",
            "Приложения и подсистемы с EXECUTION REQUIRED / System Required удерживают питание. "
            "Политика определяет, учитываются ли такие запросы в схеме.",
            "Если ПК не засыпает при «ничего не запущено» — проверьте фоновые power requests в powercfg.",
        ),
        (
            "Whether System Required power requests can keep the machine awake.",
            "Apps with execution-required requests block sleep. "
            "This policy governs how the plan honors them.",
            "If the PC won't sleep idle, inspect power requests with powercfg.",
        ),
    ),
    (
        "Allow sleep with remote opens",
        (
            "Разрешает ли сон, пока есть удалённые открытые файлы/сессии (SMB и т.п.).",
            "Запрет — клиенты не «обрываются» сном сервера/ПК с общими папками; разрешение — быстрее экономия. "
            "Влияет на домашние NAS-сценарии и общий доступ к файлам.",
            "Для файлового сервера — часто запрет от сети; для ноутбука — разрешение.",
        ),
        (
            "Whether sleep is allowed while remote file sessions are open.",
            "Blocking sleep avoids dropping SMB clients; allowing sleep saves power faster. "
            "Matters for file shares and home servers.",
            "File servers often block on AC; laptops usually allow.",
        ),
    ),
    (
        "Allow automatic wake from sleep",
        (
            "Разрешает запланированное и автоматическое пробуждение из сна (wake timers, задачи).",
            "При Allow Windows и приложения могут будить ПК для обслуживания, обновлений, медиа. "
            "При Block ноутбук не проснётся ночью от таймеров — но и задачи могут не выполниться.",
            "Отключите, если ПК сам просыпается в сумке (проверьте также wake-armed devices).",
        ),
        (
            "Allows scheduled/automatic wake from sleep (wake timers, tasks).",
            "Allow — Windows/apps may wake for maintenance, updates, media. "
            "Block — fewer surprise wakes, but missed scheduled tasks.",
            "Disable if the laptop wakes in a bag; also check wake-armed devices.",
        ),
    ),
    (
        "Legacy RTC mitigations",
        (
            "Обходные меры для старых проблем Real-Time Clock и таймеров пробуждения.",
            "Может менять, как система обрабатывает будильники RTC при переходах сон/гибернация на старом железе. "
            "На современных ПК обычно не требует вмешательства.",
            "Трогайте только при известных багах пробуждения по расписанию на legacy hardware.",
        ),
        (
            "Mitigations for legacy Real-Time Clock / wake timer issues.",
            "May alter RTC alarm handling across sleep/hibernate on older platforms. "
            "Modern systems rarely need changes.",
            "Tune only for known scheduled-wake bugs on legacy hardware.",
        ),
    ),
    (
        "Lock Console when System Wakes",
        (
            "Блокировать ли экран входа сразу после пробуждения.",
            "Require sign-in — выше безопасность в офисе и общих местах; без блокировки — быстрее вернуться к работе. "
            "Не заменяет пароль при первоначальном входе в Windows.",
            "На личном ноутбуке часто «No» от сети; в поездках — «Yes».",
        ),
        (
            "Whether the sign-in screen appears immediately after wake.",
            "Require sign-in improves security in shared spaces; skipping it speeds resume. "
            "Separate from your initial Windows sign-in policy.",
            "Personal laptops often No on AC; Yes when traveling.",
        ),
    ),
    (
        "Power Scheme Personality",
        (
            "Служебная «личность» схемы: как Windows классифицирует профиль (сбалансированная, экономия, производительность).",
            "Влияет на то, как некоторые подсистемы интерпретируют схему при автоматическом выборе политик. "
            "Обычно совпадает с типом схемы, созданной Microsoft или OEM.",
            "Проще сменить саму схему в панели электропитания, чем править этот пункт.",
        ),
        (
            "Scheme personality — how Windows classifies the plan (balanced, saver, performance).",
            "Some subsystems use it when auto-selecting defaults. "
            "Usually mirrors the plan type from Microsoft/OEM.",
            "Switching the visible power plan is simpler than editing this.",
        ),
    ),
]

for name, ru, en in _sleep_exact:
    _reg(name, ru, en)


def _processor_detail(en: str, en_l: str, sg_ru: str) -> Detail | None:
    """Разбор имён группы «процессор» — уникальное поведение по подстрокам."""

    def pr(purpose: str, behavior: str, tips: str = "") -> Detail:
        return _pack_ru(purpose, behavior, tips, en_l, en)

    if "processor performance boost mode" in en_l:
        return pr(
            "Включение и стиль Turbo Boost / Precision Boost на коротких всплесках нагрузки.",
            "Режимы вроде Enabled / Aggressive / Efficient определяют, насколько часто и долго CPU выходит за базовую частоту. "
            "Агрессивный boost — выше FPS и экспорт, больше нагрев; Efficient — стабильнее температуры, ниже пики.",
            "Игры от сети — Enabled; тихая работа от батареи — Efficient или Disabled на старых схемах."
        )
    if "processor performance boost policy" in en_l:
        return pr(
            "Политика разрешения boost относительно лимитов мощности и температуры.",
            "Отличается от boost mode: задаёт правила, когда boost разрешён (например, только одно ядро, все ядра, с учётом PL1/PL2). "
            "Жёсткая политика сглаживает пики; мягкая — даёт CPU «выстреливать» чаще.",
            "При power-limit throttling в играх смягчите политику от сети."
        )
    if "core parking max cores" in en_l:
        return pr(
            "Потолок числа логических процессоров, которые могут оставаться не «припаркованными».",
            "Ограничивает максимум активных ядер при низкой нагрузке. "
            "Ниже значение — меньше ядер онлайн, ниже idle-расход; при всплеске нагрузки Windows распаковывает ядра с задержкой.",
            "Не путать с min cores: max — верхняя граница, min — нижний пол."
        )
    if "core parking min cores" in en_l:
        return pr(
            "Минимум ядер, которые нельзя «припарковать» даже при очень низкой нагрузке.",
            "Держит больше ядер готовыми — быстрее отклик на первый поток после простоя, выше фоновое потребление. "
            "Полезно при чувствительности к latency; для автономности min часто оставляют 0 или мало.",
            "При stutter в играх от батареи попробуйте поднять min cores."
        )
    if "core parking increase policy" in en_l:
        return pr(
            "Когда распаковывать дополнительные ядра при росте нагрузки.",
            "Ideal / Single / Rocket / All cores — разная агрессивность: от одного ядра за раз до быстрого включения пакета ядер. "
            "Rocket/All быстрее реагируют на многопоточность, но дают больше скачков питания.",
            "Для рендера — агрессивнее; для офиса — Ideal."
        )
    if "core parking decrease policy" in en_l:
        return pr(
            "Когда «парковать» ядра при падении нагрузки.",
            "Определяет, насколько быстро лишние ядра уходят offline после спада CPU utilization. "
            "Быстрая парковка экономит ватты; медленная — стабильнее производительность при «рваной» нагрузке.",
            "Если FPS «пилит» при переменной нагрузке — смягчите decrease policy."
        )
    if "core parking increase time" in en_l:
        return pr(
            "Задержка перед добавлением распакованного ядра после решения increase policy.",
            "Больше время — меньше «дёрганья» числа активных ядер при кратковременных всплесках. "
            "Меньше — быстрее масштабирование вверх.",
            "Согласуйте с increase threshold: оба задают «инерцию» масштабирования."
        )
    if "core parking decrease time" in en_l:
        return pr(
            "Сколько ждать перед парковкой ядра после снижения нагрузки.",
            "Длинный интервал удерживает ядра активными «про запас» — лучше для bursty workloads. "
            "Короткий — агрессивная экономия.",
            "На ноутбуке от батареи умеренное decrease time снижает heat cycles."
        )
    if "core parking over utilization threshold" in en_l:
        return pr(
            "Порог загрузки, выше которого считается, что ядер не хватает и нужна распаковка.",
            "Ниже порог — раньше включаются дополнительные ядра; выше — дольше работа на меньшем наборе ядер.",
            "Снижение порога помогает многопоточным задачам; повышение — экономит батарею."
        )
    if "core parking concurrency threshold" in en_l:
        return pr(
            "Порог параллельной нагрузки для решений core parking.",
            "Смотрит на число одновременно busy потоков/ядер; при превышении parking ослабляется. "
            "Тонкая настройка баланса между latency и числом активных ядер.",
            "Меняйте только вместе с over utilization threshold."
        )
    if "core parking concurrency headroom threshold" in en_l:
        return pr(
            "«Запас» параллелизма перед тем, как снова ужесточать парковку.",
            "Гистерезис: после распаковки ядер система ждёт больший запас, прежде чем снова парковать — "
            "меньше осцилляций active core count.",
            "Полезно при «пиле» загрузки 50–80% в играх."
        )
    if "core parking distribution threshold" in en_l:
        return pr(
            "Как распределять нагрузку между уже распакованными ядрами vs запуск новых.",
            "Влияет на fairness и решение «дожать текущие ядра или включить ещё». "
            "Высокий порог — чаще масштабирование вширь; низкий — упор на уже активные ядра.",
            "Для серверных VM редко трогают; для desktop — экспериментально."
        )
    if "core parking utility distribution" in en_l:
        return pr(
            "Вес «полезности» ядер при выборе, какие парковать первыми.",
            "Учитывает историческую отдачу ядер (utility) — менее полезные паркуются раньше. "
            "Может улучшить производительность на NUMA/гибридных топологиях при правильных драйверах.",
            "Оставьте по умолчанию без профилирования latency."
        )
    if "core parking soft park latency" in en_l:
        return pr(
            "Мягкая задержка перед переводом ядра в parked state.",
            "Даёт окно, в которое ядро ещё может принять работу без полного offline — снижает penalty при ложных срабатываниях парковки.",
            "Увеличивайте при micro-stutter после падения нагрузки."
        )
    if "core parking parked performance state" in en_l:
        return pr(
            "Какое P-state/частотное состояние держит «припаркованное» ядро.",
            "Parked ядро не обязательно полностью off — может оставаться в глубоком idle с минимальной частотой. "
            "Режим влияет на время пробуждения vs экономию.",
            "Связано с idle demote/promote — настраивайте комплексно."
        )
    if "processor performance increase policy" in en_l:
        return pr(
            "Алгоритм повышения частоты при росте нагрузки (Ideal, Single, Rocket…).",
            "Single/Rocket быстрее наращивают MHz на одном или нескольких ядрах; Ideal плавнее. "
            "Влияет на форму кривой «нагрузка → частота», не на потолок Turbo.",
            "Для latency-sensitive — Rocket от сети."
        )
    if "processor performance decrease policy" in en_l:
        return pr(
            "Как снижать частоту, когда нагрузка падает.",
            "Aggressive decrease быстрее возвращает CPU в низкие P-states — экономия и тишина. "
            "Relaxed — держит частоту выше «про запас», меньше ramp-up после паузы.",
            "Аудио/DAW часто relaxed от сети."
        )
    if "processor performance increase threshold" in en_l:
        return pr(
            "При какой загрузке (%) менеджер питания решает поднять частоту.",
            "Ниже порог — CPU чаще считает себя «занятым» и поднимает MHz. "
            "Выше — нужна более уверенная нагрузка для ramp-up.",
            "Пара работает в паре с increase time."
        )
    if "processor performance decrease threshold" in en_l:
        return pr(
            "Загрузка, ниже которой начинается снижение частоты.",
            "Гистерезис с increase threshold предотвращает дрожание частоты на границе. "
            "Широкий зазор — стабильнее; узкий — отзывчивее.",
            "Не ставьте decrease выше increase."
        )
    if "processor performance increase time" in en_l:
        return pr(
            "Минимальное время, которое CPU ждёт перед следующим шагом повышения частоты.",
            "Короткий интервал — частые мелкие подъёмы MHz (отзывчиво, больше переключений). "
            "Длинный — реже, но крупнее шаги.",
            "Снижайте при input lag; увеличивайте для экономии от батареи."
        )
    if "processor performance decrease time" in en_l:
        return pr(
            "Задержка перед очередным шагом понижения частоты.",
            "Аналог increase time для спада: дольше — CPU остаётся «быстрым» после краткой нагрузки.",
            "Короткий decrease time быстрее охлаждает и экономит батарею."
        )
    if "processor performance time check interval" in en_l:
        return pr(
            "Как часто менеджер питания опрашивает загрузку для решений по частоте.",
            "Частый опрос — точнее реакция, чуть выше overhead в простое. "
            "Редкий — плавнее, но медленнее замечает всплески.",
            "Обычно не трогают без профилирования EPP."
        )
    if "processor performance history count" in en_l:
        return pr(
            "Сколько последних измерений загрузки учитывается при сглаживании.",
            "Больше history — инерционнее частота, меньше реакция на краткий spike. "
            "Меньше — острее отклик на bursty нагрузку.",
            "Связано с time check interval."
        )
    if "processor performance autonomous mode" in en_l:
        return pr(
            "Разрешает ли CPU/микрокод автономно выбирать частоту внутри рамок Windows.",
            "Autonomous Enabled — аппаратный HWP/CPPC чаще сам подстраивает P-states; Disabled — больше решений на стороне OS policy. "
            "На современных Intel/AMD обычно Enabled.",
            "При нестабильном boost попробуйте Disabled для диагностики."
        )
    if "processor autonomous activity window" in en_l:
        return pr(
            "Окно активности для автономного governor внутри CPU.",
            "Определяет горизонт, на котором hardware смотрит на utilization при автономном масштабировании. "
            "Короче — reactive; длиннее — smooth.",
            "Тонкий параметр HWP; оставьте default без проблем."
        )
    if "processor duty cycling" in en_l:
        return pr(
            "Duty cycling — периодическое кратковременное отключение простаивающих ядер для экономии.",
            "На поддерживаемых CPU чередует active/idle фазы на уровне ядра, снижая average power. "
            "Может добавить jitter для ultra-low-latency задач.",
            "Отключайте при проблемах DPC/audio."
        )
    if "processor energy performance preference policy" in en_l:
        return pr(
            "EPP — баланс энергии и производительности на уровне hardware (0–100 или режимы).",
            "Низкий EPP — CPU стремится к performance; высокий — к efficiency. "
            "Windows пробрасывает EPP в CPPC/HWP вместе с min/max frequency.",
            "Часто эффективнее смены min/max freq: 0 performance / 50 balanced / 100 power save."
        )
    if "processor idle disable" in en_l:
        return pr(
            "Полностью или частично отключает переходы ядер в глубокие C-states.",
            "Disable idle — ядра остаются ближе к C0 — минимальная latency пробуждения, максимальный idle drain. "
            "Используют на аудио-станциях и latency benchmarking.",
            "Не оставляйте disable от батареи."
        )
    if "processor idle demote threshold" in en_l:
        return pr(
            "Порог для «понижения» глубины idle (demote) — выход из более глубокого C-state.",
            "Когда нагрузка чуть растёт, demote возвращает ядро к более мелкому idle раньше, чем полный C0. "
            "Ниже порог — чувствительнее пробуждение из deep idle.",
            "Пара с promote threshold."
        )
    if "processor idle promote threshold" in en_l:
        return pr(
            "Порог для «повышения» глубины idle (promote) — углубление C-state при простое.",
            "Выше порог — дольше ждёт перед уходом в глубокий idle; ниже — агрессивнее экономия.",
            "При audio dropouts увеличьте promote threshold (реже deep C-states)."
        )
    if "processor idle state maximum" in en_l:
        return pr(
            "Максимально разрешённая глубина C-state для ядер.",
            "Ограничивает, насколько глубоко может «уснуть» ядро — cap на C6/C7 и т.д. "
            "Меньший max — выше idle power, ниже wake latency.",
            "Laptop gaming: иногда ограничивают max C-state от сети."
        )
    if "processor idle threshold scaling" in en_l:
        return pr(
            "Масштабирование порогов idle demote/promote под число активных ядер.",
            "Когда много ядер online, пороги могут сдвигаться, чтобы не все одновременно уходили в deep idle. "
            "Стабилизирует поведение на many-core CPU.",
            "Редко меняют вручную."
        )
    if "processor idle time check" in en_l:
        return pr(
            "Интервал проверки простоя для idle state machine.",
            "Как часто OS оценивает, можно ли углубить C-state. "
            "Чаще — точнее экономия; реже — меньше overhead.",
            "Связано с Idle Resiliency timer resolution."
        )
    if "processor idle resiliency timer resolution" in en_l:
        return pr(
            "Разрешение таймера для idle resiliency (точность учёта коротких простоев).",
            "Более г fine resolution — точнее отличает «микропростой» от реального idle, меньше ложных deep transitions. "
            "Может чуть увеличить timer interrupt rate.",
            "При DPC spikes смотрите также coalescing timeout."
        )
    if "maximum processor frequency" in en_l:
        return pr(
            "Верхний предел частоты CPU в процентах от номинала (100% = без искусственного cap).",
            "Windows не даст частоте подняться выше заданного процента даже при Turbo. "
            "Снижение cap — меньше heat/noise; 100% — полный диапазон Turbo subject to power/thermal limits.",
            "80–90% от батареи иногда спасает от throttling в тонких корпусах."
        )
    if "lower bound for processor performance throttling" in en_l:
        return pr(
            "Минимальная производительность (нижняя граница throttling) в процентах.",
            "Не даёт CPU опуститься ниже указанного процента производительности при thermal/policy throttle. "
            "Поднимает floor — стабильнее FPS под нагрузкой, выше минимальное потребление.",
            "Не путать с max frequency: throttling bound работает при перегреве."
        )
    if "upper bound for processor performance throttling" in en_l:
        return pr(
            "Максимальная производительность при throttling — потолок, выше которого OS не поднимет perf state.",
            "Используется для жёсткого ограничения peak perf (kiosk, embedded, борьба с heat). "
            "100% — без дополнительного software cap.",
            "Полезно ограничить turbo на общественных ПК."
        )
    if "system cooling policy" in en_l:
        return pr(
            "Active vs Passive охлаждение: что первично — вентилятор или снижение частоты.",
            "Active — раньше крутит fan, CPU дольше держит turbo. "
            "Passive — сначала снижает MHz, тише, но длинная нагрузка упирается в throttle.",
            "Игры от сети — Active; библиотека от батареи — Passive."
        )
    if "latency sensitivity hint" in en_l:
        if "min unparked cores" in en_l:
            return pr(
                "Минимум неприпаркованных ядер для процессов с hint «чувствительность к задержке».",
                "Игры, аудио и input stack могут помечаться latency-sensitive — Windows резервирует указанное число ядер. "
                "Больше min — стабильнее таймеры и IRQ handling, выше idle cost.",
                "Повышайте только при измеренном input lag.",
            )
        if "processor performance" in en_l:
            return pr(
                "Минимальный уровень производительности CPU для latency-sensitive потоков.",
                "Не даёт governor опускать частоту ниже порога, пока такие потоки активны. "
                "Снижает audio glitches и mouse jitter ценой батареи.",
                "Сочетается с min unparked cores/packages.",
            )
    if "initial performance for processor power efficiency class 1 when unparked" in en_l:
        return pr(
            "С какой производительности стартуют E-ядра сразу после распаковки.",
            "Высокий initial perf — E-ядра быстрее выходят на рабочую частоту; низкий — плавный ramp и меньше spike power.",
            "Для фоновых задач на E-cores — умеренный initial."
        )
    if "a floor performance for processor power efficiency class 0" in en_l:
        return pr(
            "Нижний предел perf для P-ядер, пока часть E-ядер не припаркована.",
            "Не даёт P-ядрам падать слишком низко, когда гибридный CPU держит E-ядра активными — "
            "баланс между отзывом на P и экономией на E.",
            "Только для hybrid Intel с классами эффективности."
        )
    if "processor performance level increase threshold" in en_l and "processor count increase" in en_l:
        return pr(
            "Порог для увеличения числа активных E-ядер (Class 1) при росте нагрузки.",
            "Когда многопоточная нагрузка превышает порог, Windows добавляет unparked E-cores. "
            "Ниже порог — раньше масштабирование на E; выше — дольше работа на меньшем наборе.",
            "Пара к decrease threshold для processor count."
        )
    if "processor performance level decrease threshold" in en_l and "processor count decrease" in en_l:
        return pr(
            "Порог для уменьшения числа активных E-ядер при спаде нагрузки.",
            "После падения utilization ниже порога лишние E-ядра паркуются. "
            "Гистерезис с increase threshold уменьшает flapping количества E-cores.",
            "Для battery — чуть агрессивнее decrease."
        )
    if "unparked time trigger" in en_l:
        return pr(
            "Сколько ядро должно оставаться распакованным минимум после решения unpark.",
            "Предотвращает мгновенную повторную парковку при кратком всплеске — стабилизирует core count.",
            "Увеличивайте при oscillating core parking."
        )
    if "allow throttle states" in en_l:
        return pr(
            "Разрешает ли схема переходы CPU в throttle states (T-states) помимо P-states.",
            "Block throttle — CPU реже использует аппаратное снижение тактов внутри частоты; Allow — полный набор состояний. "
            "Влияет на fine-grained power на некоторых платформах.",
            "Обычно Allow; Block — niche tuning."
        )
    return None


def explain_ru(en: str, subgroup: str, sg_name: str) -> Detail:
    en_c = re.sub(r"^[-\d;,]+", "", en or "").strip()
    en_l = en_c.lower()
    key = en_l

    if key in _EXACT_RU:
        return _EXACT_RU[key]

    proc = _processor_detail(en_c, en_l, sg_name)
    if proc:
        return proc

    # Display / video
    if en_l == "display brightness":
        return _pack_ru(
            "Яркость встроенной панели в процентах для данной схемы.",
            "Отдельные значения для сети и батареи; Windows применяет при переключении источника питания. "
            "Не заменяет аппаратные клавиши яркости — задаёт baseline схемы.",
            "Часто дублируется с «яркость при затемнении» — согласуйте оба.",
            en_l,
        )
    if en_l == "dim display after":
        return _pack_ru(
            "Через сколько бездействия экран переходит на пониженную яркость (не выключение).",
            "Промежуточный шаг перед turn-off: снижает расход до полного гашения панели. "
            "0 — функция отключена, сразу только timeout выключения.",
            "Полезно на OLED для снижения burn-in risk при статичном UI.",
            en_l,
        )
    if en_l == "display dim brightness":
        return _pack_ru(
            "Уровень яркости в режиме dim (между полной и выключенной).",
            "Процент от максимума; применяется после «Dim display after». "
            "Слишком низкий dim мешает читать; слишком высокий — слабая экономия.",
            "15–40% типично от батареи.",
            en_l,
        )
    if en_l == "display brightness weight":
        return _pack_ru(
            "Вес яркости при автоматическом балансе нескольких источников (сенсор, схема, пользователь).",
            "Больший weight — сильнее влияние этого параметра на итоговую яркость при adaptive algorithms. "
            "Редко меняется вручную на consumer hardware.",
            "При странной adaptive brightness проверьте sensor vs scheme weights.",
            en_l,
        )
    if en_l == "enable adaptive brightness":
        return _pack_ru(
            "Включает автоматическую подстройку яркости по датчику освещённости.",
            "On — система поднимает/опускает backlight в помещении и на улице. "
            "Off — только ручная и scheme brightness.",
            "Отключите, если яркость «прыгает» при PWM/мигании LED.",
            en_l,
        )
    if en_l == "console lock display off timeout":
        return _pack_ru(
            "Когда гасить дисплей при заблокированной консоли (Win+L / lock screen).",
            "Отдельно от общего display timeout — управляет backlight при уже показанном lock screen. "
            "Короткий timeout экономит батарею на этапе «заблокирован, но включён».",
            "0 — не гасить до общего таймера.",
            en_l,
        )
    if en_l == "video playback quality bias":
        return _pack_ru(
            "Приоритет качества vs экономии при декодировании видео в плеере Windows.",
            "Quality bias — выше bitrate/resolution tolerance, больше GPU/CPU; power bias — снижение качества ради батареи.",
            "На просмотре offline от батареи — power; от сети — quality.",
            en_l,
        )
    if en_l == "advanced color quality bias":
        return _pack_ru(
            "Баланс точности цвета/HDR vs энергопотребления при обработке кадра.",
            "Quality — глубже color processing; power — упрощённые пути в GPU/display pipeline.",
            "Для HDR контента на питании — quality.",
            en_l,
        )
    if en_l == "when playing video":
        return _pack_ru(
            "Поведение схемы, пока система считает, что идёт воспроизведение видео.",
            "Может удерживать display awake, ослаблять sleep timers или менять GPU clock — зависит от значения enum. "
            "Предотвращает затемнение посреди фильма.",
            "Согласуйте с «Allow display required» и multimedia policies.",
            en_l,
        )
    if en_l == "when sharing media":
        return _pack_ru(
            "Поведение при трансляции/расширении медиа (Miracast, DLNA, второй экран).",
            "Часто запрещает сон и display off, пока идёт sharing session. "
            "Off — сон возможен даже при casting (может оборвать сессию).",
            "Для презентаций — optimize for session.",
            en_l,
        )
    if "video power down timeout" in en_l:
        return _pack_ru(
            "Таймаут выключения видеоподсистемы/дисплея после окончания активного video pipeline.",
            "Отличается от общего «Turn off display» — касается GPU/display engine idle после playback.",
            "Короткий — быстрее экономия после закрытия плеера.",
            en_l,
        )

    # USB
    if en_l == "hub selective suspend timeout":
        return _pack_ru(
            "Секунды простоя USB-хаба перед selective suspend.",
            "После таймера хаб отключает питание неиспользуемых портов. "
            "Короткий — агрессивная экономия; длинный — стабильнее периферия.",
            "При отвалах мыши/клавиатуры увеличьте timeout от сети.",
            en_l,
        )
    if "usb 3 link power mangement" in en_l or "usb 3 link power management" in en_l:
        return _pack_ru(
            "U1/U2/U3 link states для USB 3 — экономия на физическом линке.",
            "Maximum power savings — глубокие U-states; minimum — линк бодрствует, выше расход на idle портах.",
            "Док-станции с NVMe иногда требуют moderate/off от сети.",
            en_l,
        )
    if en_l == "setting ioc on all tds":
        return _pack_ru(
            "IOC (Interrupt On Completion) на всех transfer descriptors USB.",
            "Меньше прерываний при bulk transfer — ниже CPU wake rate; может добавить latency completion на старых контроллерах.",
            "Трогайте при профилировании USB storage latency.",
            en_l,
        )

    # PCI
    if en_l == "link state power management":
        return _pack_ru(
            "ASPM и состояния PCIe link (L0s/L1) для устройств на шине.",
            "Maximum — агрессивный ASPM, минимальный idle power; Off — линк всегда готов, лучше для dGPU/NVMe latency.",
            "Настольный ПК от сети часто Off; ноутбук — Moderate/Maximum от батареи.",
            en_l,
        )

    # Battery
    if en_l == "critical battery notification":
        return _pack_ru(
            "Показывать ли уведомление при достижении критического уровня заряда.",
            "On — всплывающее предупреждение перед принудительным действием (sleep/shutdown). "
            "Off — тишина, но риск внезапного отключения выше.",
            "Оставляйте On для ноутбуков.",
            en_l,
        )
    if en_l == "low battery notification":
        return _pack_ru(
            "Уведомление о низком заряде (раньше критического).",
            "Даёт время подключить зарядку или сохранить работу. "
            "Порог задаётся отдельными trip point settings.",
            "Не отключайте на мобильных устройствах.",
            en_l,
        )
    if en_l == "low battery warning trip point":
        return _pack_ru(
            "Процент заряда для первого предупреждения «низкий заряд».",
            "Выше значение — раньше предупреждение, меньше риск deep discharge; ниже — дольше работа без розетки.",
            "10–20% типично.",
            en_l,
        )
    if en_l == "reserve battery warning trip point":
        return _pack_ru(
            "Резервный порог: ниже него Windows может ограничивать фон и показывать reserve warning.",
            "Защищает химию батареи от глубокого разряда; может включать energy saver behaviors.",
            "OEM иногда поднимает reserve на corporate laptops.",
            en_l,
        )
    if en_l == "charge level":
        return _pack_ru(
            "Целевой или ограничивающий уровень заряда (OEM/battery preservation).",
            "На поддерживаемых ноутбуках cap 80% prolongs cycle life; 100% — полная ёмкость.",
            "Зависит от OEM software — не все системы применяют через powercfg.",
            en_l,
        )
    if en_l == "first low battery alarm action":
        return _pack_ru(
            "Действие при первом alarm уровне (обычно уведомление или sleep).",
            "Do nothing — только UI; Sleep/Hibernate — принудительная защита данных.",
            "Согласуйте с trip points.",
            en_l,
        )
    if en_l == "second low battery alarm action":
        return _pack_ru(
            "Действие при втором, более жёстком alarm уровне.",
            "Часто Hibernate или Shutdown для сохранения работы перед полным отключением.",
            "Second action должно быть «жёстче» first.",
            en_l,
        )
    if en_l == "first power trip point":
        return _pack_ru(
            "Первый порог мощности/заряда для alarm pipeline (OEM-specific semantics).",
            "Срабатывает раньше critical — триггер для уведомлений или ограничения perf.",
            "Документируйте вместе с low battery trip points.",
            en_l,
        )
    if en_l == "enable forced shutdown":
        return _pack_ru(
            "Разрешает принудительное выключение при исчерпании батареи.",
            "On — система выключается на critical level вместо бесконечного deep sleep attempt. "
            "Off — риск зависания на 0%.",
            "On для данных важнее «дожать» последние минуты.",
            en_l,
        )

    # Buttons
    if en_l == "power button action":
        return _pack_ru(
            "Действие при коротком/стандартном нажатии кнопки питания.",
            "Sleep — быстрый сон; Hibernate — на диск; Shut down — полное выключение; Do nothing — игнор (редко).",
            "На ATX часто Sleep; на ноутбуке — Sleep или Hibernate.",
            en_l,
        )
    if en_l == "sleep button action":
        return _pack_ru(
            "Действие отдельной кнопки сна (если есть на корпусе/клавиатуре).",
            "Обычно Sleep; можно назначить Hibernate для «глубокого» сна без hybrid shutdown.",
            "Отдельно от power button — настройте обе.",
            en_l,
        )
    if en_l == "user interface sleep button action":
        return _pack_ru(
            "Действие кнопки «Сон» в меню «Пуск» / UI (не аппаратной).",
            "Может отличаться от hardware sleep button — например UI → Hibernate, крышка → Sleep.",
            "Полезно для согласованности UX.",
            en_l,
        )
    if en_l == "lid-close action":
        return _pack_ru(
            "Что делать при закрытии крышки ноутбука.",
            "Sleep — классика; Do nothing — работа с закрытой крышкой на док-станции; Shut down — жёстко.",
            "Dock + внешний монитор: Do nothing от сети, Sleep от батареи.",
            en_l,
        )
    if en_l == "lid open action":
        return _pack_ru(
            "Действие при открытии крышки после sleep.",
            "Wake — сразу пробуждение; Nothing — остаётся спать до кнопки.",
            "Wake удобнее для быстрого resume.",
            en_l,
        )

    # Idle resiliency / energy
    if en_l == "io coalescing timeout":
        return _pack_ru(
            "Окно объединения мелких I/O для снижения частоты пробуждений CPU.",
            "Дольше timeout — меньше wakeups, возможна чуть большая latency disk/network. "
            "Короче — responsive, выше idle power.",
            "При DPC latency уменьшайте осторожно.",
            en_l,
        )
    if en_l == "execution required power request timeout":
        return _pack_ru(
            "Сколько удерживается Execution Required request без активности приложения.",
            "По истечении система может игнорировать stale request и разрешить sleep. "
            "Короткий — быстрее сон после зависшего приложения.",
            "Если ПК не спит — не увеличивайте без проверки виновника.",
            en_l,
        )
    if en_l == "specifies deep sleep":
        return _pack_ru(
            "Разрешает или задаёт параметры deep sleep platform idle (S0 low-power idle).",
            "Connected Modern Standby использует deep sleep для фоновой сети при выключенном экране. "
            "Disable — ближе к classic idle, выше drain.",
            "Зависит от поддержки firmware.",
            en_l,
        )
    if en_l == "enable adaptive powerdown timeout.":
        return _pack_ru(
            "Адаптивный таймаут powerdown для idle resiliency subsystem.",
            "Подстраивает timeout под паттерн активности — дольше awake после недавней I/O.",
            "Оставьте default без проблем wake.",
            en_l,
        )
    if en_l == "energy saver policy":
        return _pack_ru(
            "Когда включается режим экономии энергии Windows (Energy Saver).",
            "Может снижать яркость, фон, turbo при низком заряде или вручную. "
            "Aggressive — раньше и сильнее ограничения.",
            "Связано с battery reserve trip points.",
            en_l,
        )
    if en_l == "device idle policy":
        return _pack_ru(
            "Политика простоя устройств (Device Idle): когда периферия может уходить в D3cold/D2.",
            "Performance — реже deep device idle; Power savings — агрессивнее D-state transitions.",
            "При USB/NIC dropouts смягчите от сети.",
            en_l,
        )
    if en_l == "disconnected standby mode":
        return _pack_ru(
            "Поведение Modern Standby при отключении от сети (battery disconnected scenarios / OEM).",
            "Может ограничивать фоновую сеть и sync для сохранения заряда в CS mode.",
            "OEM-specific; меняйте по документации производителя.",
            en_l,
        )
    if "networking connectivity in standby" in en_l:
        return _pack_ru(
            "Остаётся ли сеть активной в Modern Standby (Connected vs Disconnected).",
            "Connected — фоновые уведомления и sync в сне; Disconnected — экономия, но задержка push.",
            "«Maintain» vs «Disable» критично для ноутбуков в сумке (heat/drain).",
            en_l,
        )
    if en_l == "standby budget percent":
        return _pack_ru(
            "Доля (%) допустимой фоновой активности в Connected Standby до ужесточения лимитов.",
            "Чем выше процент — тем больше CPU/сеть/Radio работы разрешено в low-power idle; ниже — жёстче caps и дольше батарея.",
            "OEM/CS tuning; без проблем автonomности не меняйте.",
            en_l,
            en_c,
        )
    if en_l == "standby budget grace period":
        return _pack_ru(
            "Стартовое «льготное» окно после входа в standby, когда budget percent ещё не режет фон.",
            "Даёт приложениям время завершить короткий sync сразу после гашения экрана; по окончании grace действует основной budget.",
            "Связано с Standby Budget Percent.",
            en_l,
            en_c,
        )
    if en_l == "standby reserve time":
        return _pack_ru(
            "Сколько времени система удерживает «резерв» фоновой активности в standby перед clamp.",
            "Пока reserve не исчерпан, Windows мягче относится к wakeups; после — агрессивнее гасит фон.",
            "Тонкий параметр Modern Standby.",
            en_l,
            en_c,
        )
    if en_l == "standby reserve grace period":
        return _pack_ru(
            "Grace period для standby reserve — задержка перед началом отсчёта reserve time.",
            "Отличается от budget grace: касается второго уровня лимитов после исчерпания budget percent.",
            "Для диагностики CS drain используйте powercfg /sleepstudy.",
            en_l,
            en_c,
        )

    # IRQ / Intel graphics
    if en_l == "interrupt steering mode":
        return _pack_ru(
            "Режим распределения аппаратных прерываний по ядрам.",
            "Default / static / dynamic — влияет на то, фиксируются ли IRQ на CPU0 или балансируются для load.",
            "При DPC latency пробуйте static на последнем ядре (осторожно).",
            en_l,
        )
    if en_l == "interrupt steering target load for each processor":
        return _pack_ru(
            "Целевая загрузка ядра при dynamic interrupt steering.",
            "IRQ мигрируют к ядрам ниже target load — выравнивание без перегруза CPU0.",
            "Согласуйте с режимом steering mode.",
            en_l,
        )
    if en_l == "intel(r) graphics power plan":
        return _pack_ru(
            "План питания встроенного GPU Intel (частоты, render idle, media blocks).",
            "Max performance — выше clocks iGPU при лёгкой 3D; battery — агрессивный gfx idle.",
            "При switchable graphics основная нагрузка может быть на dGPU.",
            en_l,
        )
    if en_l == "gpu preference policy":
        return _pack_ru(
            "Предпочтение iGPU vs dGPU для приложений в рамках power scheme (связано с hybrid graphics).",
            "Power saving — чаще iGPU; performance — dGPU раньше включается.",
            "Дублирует часть Windows Graphics settings per-app.",
            en_l,
        )

    # Presence
    if en_l == "non-sensor input presence timeout":
        return _pack_ru(
            "Таймаут без ввода для presence detection без аппаратного сенсора.",
            "По истечении система считает пользователя отсутствующим — могут сработать dim/display/sleep раньше.",
            "Короче — агрессивнее экономия в офисе.",
            en_l,
        )
    if en_l == "user presence prediction mode":
        return _pack_ru(
            "Использование ML/эвристик для предсказания присутствия пользователя.",
            "Enabled — Windows может задерживать sleep если ожидает возвращение; Disabled — только таймеры.",
            "Privacy/accuracy tradeoff на новых builds.",
            en_l,
        )

    # Allow display required
    if en_l == "allow display required policy":
        return _pack_ru(
            "Учитывает ли схема Display Required power requests (удержание экрана включённым).",
            "Allow — презентации и некоторые apps могут блокировать turn-off display. "
            "Block — таймеры дисплея срабатывают независимо.",
            "Если экран не гаснет — проверьте display required requests.",
            en_l,
        )

    # Fallback — уникально за счёт полного имени и раздела
    return _pack_ru(
        f"Параметр раздела «{sg_name}» в схеме электропитания.",
        f"Конкретное имя в Windows: «{en_c}». "
        f"Значение меняет одну грань политики «{sg_name}» (таймер, порог, режим или разрешение) — "
        f"эффект проявляется при смене нагрузки, источника питания или состояния простоя; "
        f"точная семантика зависит от версии Windows и OEM-драйверов.",
        "При сомнении сравните «От сети» и «От батареи» в стандартном окне дополнительных параметров.",
        en_l,
    )


def _processor_detail_en(en_c: str, en_l: str) -> Detail | None:
    """English texts for processor settings (same match order as _processor_detail)."""
    rules: list[tuple[str, Detail]] = [
        (
            "processor performance boost mode",
            (
                "Turbo Boost style for short load spikes.",
                "Enabled/Aggressive modes raise peak frequency more often; Efficient/Disabled flattens peaks and heat.",
                "Games on AC — Enabled; quiet battery — Efficient.",
            ),
        ),
        (
            "processor performance boost policy",
            (
                "When Turbo is allowed relative to power/thermal limits.",
                "Stricter policy smooths spikes; permissive policy lets the CPU burst more often.",
                "If games hit PL1, ease policy on AC.",
            ),
        ),
        (
            "core parking max cores",
            (
                "Maximum unparked logical processors.",
                "A lower cap keeps fewer cores online in idle (saves power); load bursts may wait for unparking.",
                "Not the same as min cores.",
            ),
        ),
        (
            "core parking min cores",
            (
                "Minimum cores that must stay unparked.",
                "A higher floor improves latency after idle at the cost of background power.",
                "Try raising min cores if battery gaming stutters.",
            ),
        ),
        (
            "core parking increase policy",
            (
                "When to unpark additional cores as load rises.",
                "Ideal/Single/Rocket/All cores — different aggressiveness scaling core count.",
                "Rendering — more aggressive; office — Ideal.",
            ),
        ),
        (
            "core parking decrease policy",
            (
                "When to park cores as load falls.",
                "Faster parking saves watts; slower parking stabilizes bursty workloads.",
                "Ease decrease policy if FPS oscillates.",
            ),
        ),
        (
            "core parking increase time",
            (
                "Delay before adding another unparked core.",
                "Longer delay reduces core-count flapping on brief spikes.",
                "Tune with increase threshold.",
            ),
        ),
        (
            "core parking decrease time",
            (
                "Delay before parking a core after load drops.",
                "Longer delay keeps spare cores ready; shorter delay saves battery.",
                "Moderate values on battery reduce heat cycles.",
            ),
        ),
        (
            "core parking over utilization threshold",
            (
                "Utilization level that triggers unparking.",
                "Lower threshold unparks sooner; higher threshold keeps fewer cores longer.",
                "Lower for multithreaded work; higher for battery.",
            ),
        ),
        (
            "processor performance increase policy",
            (
                "Algorithm for raising frequency when load increases.",
                "Single/Rocket ramp MHz faster than Ideal on one or more cores.",
                "Rocket on AC for latency-sensitive apps.",
            ),
        ),
        (
            "processor performance decrease policy",
            (
                "How quickly frequency drops when load falls.",
                "Aggressive decrease saves power; relaxed keeps MHz higher for faster recovery.",
                "DAW/audio often relaxed on AC.",
            ),
        ),
        (
            "processor performance increase threshold",
            (
                "Utilization % that triggers a frequency increase.",
                "Lower threshold — CPU ramps up sooner; higher — needs heavier load.",
                "Works as a pair with increase time.",
            ),
        ),
        (
            "processor performance decrease threshold",
            (
                "Utilization % that triggers frequency decrease.",
                "Hysteresis with increase threshold reduces MHz oscillation.",
                "Keep decrease below increase.",
            ),
        ),
        (
            "processor performance increase time",
            (
                "Wait time between upward frequency steps.",
                "Shorter — snappier ramps; longer — fewer P-state transitions.",
                "Shorten for input lag; lengthen for battery.",
            ),
        ),
        (
            "processor performance decrease time",
            (
                "Wait time between downward frequency steps.",
                "Longer — CPU stays fast after brief load; shorter — cools and saves faster.",
                "",
            ),
        ),
        (
            "processor energy performance preference policy",
            (
                "EPP — energy vs performance preference (CPPC/HWP).",
                "Lower EPP favors performance; higher favors efficiency within min/max frequency bounds.",
                "Often more effective than tweaking min/max alone.",
            ),
        ),
        (
            "maximum processor frequency",
            (
                "Maximum CPU frequency cap (% of nominal).",
                "Below 100% artificially limits Turbo; 100% allows full hardware range subject to thermals.",
                "Try 80–90% on battery in thin laptops.",
            ),
        ),
        (
            "system cooling policy",
            (
                "Active vs passive cooling priority.",
                "Active spins fans sooner and holds Turbo longer; passive lowers MHz first (quieter, more throttle risk).",
                "Active on AC for games; passive on battery for quiet work.",
            ),
        ),
        (
            "processor idle disable",
            (
                "Disables deep CPU idle (C-states).",
                "Keeps cores nearer C0 — lowest wake latency, highest idle power.",
                "Never disable on battery.",
            ),
        ),
    ]
    for sub, detail in rules:
        if sub in en_l:
            return _pack_en(*detail, en_l)
    if _processor_detail(en_c, en_l, "processor"):
        return _pack_en(
            f"Processor setting: {en_c}",
            f"“{en_c}” fine-tunes Windows CPU power management (parking, frequency, idle, or boost). "
            f"Values apply separately for On battery and Plugged in; effects depend on your CPU generation and drivers.",
            "Change one setting at a time and compare AC vs battery.",
            en_l,
        )
    return None


def _misc_detail_en(en_c: str, en_l: str, sg_name: str) -> Detail | None:
    """English texts for selected non-processor settings (exact Windows names)."""
    named: dict[str, Detail] = {
        "display brightness": (
            "Built-in panel brightness (%).",
            "Separate On battery vs Plugged in baselines; hardware brightness keys still apply.",
            "Keep consistent with dim brightness.",
        ),
        "dim display after": (
            "Idle time before the panel dims (before turn-off).",
            "Intermediate step before display off; 0 disables dimming.",
            "Useful on OLED laptops.",
        ),
        "hub selective suspend timeout": (
            "USB hub idle seconds before selective suspend.",
            "Shorter timeout saves power; peripherals may need replug after deep suspend.",
            "Raise timeout if mouse/keyboard drop on battery.",
        ),
        "link state power management": (
            "PCIe ASPM / link low-power states.",
            "Maximum savings lower idle watts; Off keeps links ready for GPU/NVMe latency.",
            "Desktops on AC often Off; laptops Moderate on battery.",
        ),
    }
    if en_l in named:
        return _pack_en(*named[en_l], en_l)
    return None


def explain_en(en: str, subgroup: str, sg_name: str) -> Detail:
    en_c = re.sub(r"^[-\d;,]+", "", en or "").strip()
    en_l = en_c.lower()
    if en_l in _EXACT_EN:
        return _EXACT_EN[en_l]
    proc = _processor_detail_en(en_c, en_l)
    if proc:
        return proc
    misc = _misc_detail_en(en_c, en_l, sg_name)
    if misc:
        return misc
    return _pack_en(
        f"A setting in the “{sg_name}” section of the power plan.",
        f"Windows name: “{en_c}”. "
        f"This value adjusts one facet of “{sg_name}” policy — "
        f"effects show up under load changes, AC/DC switch, or idle.",
        "When unsure, compare On battery vs Plugged in in advanced power settings.",
        en_l,
    )
