# Power Settings Explorer

[![CI](https://github.com/WtekSupport/PowerSettingsExplorer/actions/workflows/ci.yml/badge.svg)](https://github.com/WtekSupport/PowerSettingsExplorer/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/WtekSupport/PowerSettingsExplorer?include_prereleases)](https://github.com/WtekSupport/PowerSettingsExplorer/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Windows 10 / 11** utility to show or hide entries in  
**Power Options → Change plan settings → Change advanced power settings**.

Built in **Rust** with **zero crates.io dependencies** (`std` + Win32 FFI only).  
Changes **visibility only** — never AC/DC timeout or policy values.

---

## Features

- Live tree of subgroups and settings from the Windows registry
- **English / Russian** UI — switch anytime; selection and expand state are kept
- Sectioned descriptions (Purpose / Behavior / In practice / …)
- Visibility: **On** · **Off** · **Auto**
  - **On** — show in Advanced power settings
  - **Off** — hide (`POWER_ATTRIBUTE_HIDE`)
  - **Auto** — remove `Attributes` (Windows default)
- Requests administrator rights on launch (UAC)

## Download

1. Open [Releases](https://github.com/WtekSupport/PowerSettingsExplorer/releases/latest).
2. Download `power-settings-explorer.exe`.
3. Run it and accept the UAC prompt.

## Requirements

| | |
|---|---|
| OS | Windows 10 / 11 x64 |
| Privileges | Administrator (prompted automatically) |
| Build | [Rust stable, MSVC toolchain](https://www.rust-lang.org/tools/install) |

## Build from source

```powershell
git clone https://github.com/WtekSupport/PowerSettingsExplorer.git
cd PowerSettingsExplorer
cargo build --release
```

Binary: `target\release\power-settings-explorer.exe`

Or: `.\build.ps1`

UAC elevation is configured via [`.cargo/config.toml`](.cargo/config.toml)  
(`/MANIFESTUAC:level='requireAdministrator'`). [`app.manifest`](app.manifest) is a reference copy.

## Usage

1. Launch the `.exe` and approve UAC.
2. Select a section or setting in the left tree.
3. Read the description on the right.
4. For a setting, choose **On / Off / Auto** — written to the registry immediately.
5. Close and reopen the Windows **Advanced power settings** dialog to see the result.

## Safety

- Prefer a system restore point or registry export before bulk edits.
- The app mainly sets or deletes the `Attributes` DWORD under existing PowerSettings keys.
- AC/DC values are left untouched.

See [SECURITY.md](SECURITY.md).

## Project layout

```text
src/
  main.rs              Entry + elevation
  i18n.rs              EN / RU UI strings
  power/               Scan, visibility, bilingual catalog
  win/                 Win32 FFI, UI, registry, powrprof
scripts/               Catalog generator, git hooks
.github/workflows/     CI and release
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). PRs welcome — please keep the **no external crates** policy.

## License

[MIT](LICENSE)

---

## По-русски

**Power Settings Explorer** — утилита для **Windows 10/11**: показывает или скрывает пункты в  
**Параметры электропитания → Настройка схемы → Изменить дополнительные параметры**.

- Селектор **Вкл / Выкл / Авто** — только видимость, не значения «От сети» / «От батареи»
- Интерфейс **русский / English**, описания по блокам
- Запуск от администратора (UAC)
- Без зависимостей с crates.io

Скачать: [Releases](https://github.com/WtekSupport/PowerSettingsExplorer/releases/latest) → `power-settings-explorer.exe`  
Сборка: `cargo build --release` → `target\release\power-settings-explorer.exe`
