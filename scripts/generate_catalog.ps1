# Generates src/power/catalog.rs from settings_export.csv
$ErrorActionPreference = 'Stop'
$csv = Import-Csv (Join-Path $PSScriptRoot '..\settings_export.csv')
$out = Join-Path $PSScriptRoot '..\src\power\catalog.rs'

function Escape-Rust([string]$s) {
    $s = $s -replace '\\', '\\\\'
    $s = $s -replace '"', '\"'
    $s = $s -replace '\r', ''
    $s
}

function Title-Ru([string]$en, [string]$guid) {
    $g = $guid.ToLower()
    $map = @{
        '4faab71a-92e5-4726-b531-224559672d19' = 'Политика простоя устройств'
        '68afb2d9-ee95-47a8-8f50-4115088073b1' = 'Режим ожидания при отключении от сети'
        'f15576e8-98b7-4186-b944-eafa664402d9' = 'Сеть в режиме сна'
        '6738e2c4-e8a5-4a42-b16a-e040e769756e' = 'Отключение жёсткого диска через'
        '0e796bdb-100d-47d6-a2d5-f7d2daa51f51' = 'Требовать пароль при пробуждении'
        '245d8541-3943-4422-b025-13a784f679b7' = 'Тип схемы электропитания'
        'be337238-0d82-4146-a960-4f3749d470c7' = 'Режим ускорения процессора'
    }
    if ($map.ContainsKey($g)) { return $map[$g] }
    $en = ($en -replace '^[-\d;,]+', '').Trim()
    if ([string]::IsNullOrWhiteSpace($en)) { return "Параметр питания ($g)" }
    # crude EN->RU for common words
    $t = $en
    $repl = @{
        'timeout' = 'таймаут'
        'Hard disk' = 'Жёсткий диск'
        'Harddisk' = 'Жёсткий диск'
        'Sleep' = 'Сон'
        'System' = 'Система'
        'Processor' = 'Процессор'
        'USB' = 'USB'
        'Power' = 'Питание'
        'button' = 'кнопка'
        'action' = 'действие'
        'Display' = 'Экран'
        'Minimum' = 'Минимум'
        'Maximum' = 'Максимум'
        'performance' = 'производительность'
        'Idle' = 'Простой'
        'Link State Power Management' = 'Управление питанием канала PCIe'
        'NVMe' = 'NVMe'
        'AHCI' = 'AHCI'
        'Intel' = 'Intel'
        'Graphics' = 'графика'
        'Multimedia' = 'Мультимedia'
        'Battery' = 'Батарея'
        'Video' = 'Видео'
        'Interrupt' = 'Прерывания'
        'Networking connectivity in Standby' = 'Сетевое подключение в режиме ожидания'
        'Device idle policy' = 'Политика простоя устройств'
        'Disconnected Standby Mode' = 'Режим ожидания при отключении'
    }
    foreach ($k in $repl.Keys) { $t = $t -replace [regex]::Escape($k), $repl[$k] }
    if ($t -eq $en) { "Параметр: $en" } else { $t }
}

function Body-Ru([string]$en, [string]$guid) {
    $g = $guid.ToLower()
    $custom = @{
        '4faab71a-92e5-4726-b531-224559672d19' = 'Управляет тем, переводить ли устройства (USB, сеть и др.) в экономичный режим, пока компьютер работает. «Производительность» — меньше задержек при подключении устройств; «Энергосбережение» — дольше автономная работа.'
        '68afb2d9-ee95-47a8-8f50-4115088073b1' = 'Когда ноутбук на батарее и отключён от Wi‑Fi/сети, Windows может глубже усыплять компоненты. «Обычный» — баланс; «Агрессивный» — максимальная экономия, возможны задержки при выходе из сна.'
        'f15576e8-98b7-4186-b944-eafa664402d9' = 'Разрешает ли компьютеру оставаться в сети в режиме Modern Standby (получать уведомления, обновления). «Отключить» экономит заряд; «Включить» — удобнее для мессенджеров и почты.'
        '6738e2c4-e8a5-4a42-b16a-e040e769756e' = 'Через сколько секунд бездействия диск может отключаться для экономии энергии. 0 = не отключать. Не влияет на видимость пункта — только на поведение схемы.'
        '0e796bdb-100d-47d6-a2d5-f7d2daa51f51' = 'Нужно ли вводить пароль Windows после выхода из сна или гибернации. Важно для безопасности на чужих компьютерах.'
        '245d8541-3943-4422-b025-13a784f679b7' = 'Определяет базовый характер схемы: экономия, баланс или высокая производительность. Обычно задаётся при создании схемы OEM (например, MSI optimised).'
        'be337238-0d82-4146-a960-4f3749d470c7' = 'Как агрессивно процессор повышает частоту (Turbo Boost). «Агрессивный» — выше FPS и отклик; «Эффективный» — тише и холоднее ноутбук.'
        '5ca83367-6e45-459f-a27b-476b1d01c936' = 'Что делать при закрытии крышки ноутбука: сон, гибернация, выключение или ничего.'
        '7648efa3-dd9c-4e3e-b566-50f929386280' = 'Действие при нажатии кнопки питания на корпусе.'
        '96996bc0-ad50-47ec-923b-6f41874dd9eb' = 'Действие при нажатии кнопки сна (если есть на клавиатуре или корпусе).'
        '29f6c1db-86da-48c5-9fdb-f2b67b1f44da' = 'Через сколько минут бездействия компьютер перейдёт в режим сна.'
        '9d7815a6-7ee4-497e-8888-515a05f02364' = 'Через сколько минут бездействия компьютер уйдёт в гибернацию (полное сохранение на диск).'
        'ee12f906-d277-404b-b6da-e5fa1a576df5' = 'Экономия энергии на шине PCI Express (видеокарта, NVMe). «Максимальная экономия» может чуть снижать производительность диска/GPU.'
    }
    if ($custom.ContainsKey($g)) { return $custom[$g] }
    $en = ($en -replace '^[-\d;,]+', '').Trim()
    if ([string]::IsNullOrWhiteSpace($en)) {
        return 'Скрытый или редкий параметр производителя/драйвера. Изменение видимости только добавляет или убирает пункт в окне «Дополнительные параметры»; значения схемы при этом не меняются.'
    }
    return "Отвечает за настройку «$en» в схеме электропитания. Показ или скрытие пункта влияет только на то, видите ли вы его в «Параметры электропитания → Настройка схемы → Дополнительные параметры». Сами значения «От сети» / «От батареи» приложение не трогает."
}

$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine('// Auto-generated by scripts/generate_catalog.ps1 — do not edit by hand unless re-run script.')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('pub struct CatalogEntry {')
[void]$sb.AppendLine('    pub guid: &'static str,')
[void]$sb.AppendLine('    pub title_ru: &'static str,')
[void]$sb.AppendLine('    pub body_ru: &'static str,')
[void]$sb.AppendLine('}')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('pub static ENTRIES: &[CatalogEntry] = &[')

$seen = @{}
foreach ($row in $csv) {
    $guid = $row.Setting.ToLower()
    if ($seen.ContainsKey($guid)) { continue }
    $seen[$guid] = $true
    $title = Escape-Rust (Title-Ru $row.EN $guid)
    $body = Escape-Rust (Body-Ru $row.EN $guid)
    [void]$sb.AppendLine('    CatalogEntry {')
    [void]$sb.AppendLine("        guid: `"$guid`",")
    [void]$sb.AppendLine("        title_ru: `"$title`",")
    [void]$sb.AppendLine("        body_ru: `"$body`",")
    [void]$sb.AppendLine('    },')
}
[void]$sb.AppendLine('];')
[void]$sb.AppendLine('')
[void]$sb.AppendLine('pub fn lookup(setting_guid: &str) -> Option<&''CatalogEntry> {')
[void]$sb.AppendLine('    let g = setting_guid.to_lowercase();')
[void]$sb.AppendLine('    ENTRIES.iter().find(|e| e.guid == g)')
[void]$sb.AppendLine('}')

Set-Content -Path $out -Value $sb.ToString() -Encoding UTF8
Write-Host "Wrote $($seen.Count) entries to $out"
