# Power Settings Visibility

[![CI](https://github.com/OWNER/power-settings-visibility/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/power-settings-visibility/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Windows 10 / 11** utility that shows or hides items in  
> **Power Options → Change plan settings → Change advanced power settings**.

Built in **Rust with zero crates.io dependencies** (`std` + Win32 FFI only).  
Does **not** change AC/DC values — only the **visibility** of each setting.

---

## Features

- Tree of subgroups and settings from the live registry
- Russian titles and sectioned descriptions (Назначение / Поведение / На практике / …)
- Visibility selector: **Вкл** · **Выкл** · **Авто**
  - **Вкл** — show in Advanced power settings (`Attributes` without HIDE)
  - **Выкл** — hide (`POWER_ATTRIBUTE_HIDE`)
  - **Авто** — remove `Attributes` (Windows default)
- Requests administrator rights on launch (UAC)

## Screenshots

*(Add screenshots under `docs/` after publishing, then link them here.)*

## Requirements

| | |
|---|---|
| OS | Windows 10 / 11 x64 |
| Privileges | Administrator (prompted automatically) |
| Build | [Rust stable, MSVC toolchain](https://www.rust-lang.org/tools/install) |

## Install (release binary)

1. Open [Releases](https://github.com/OWNER/power-settings-visibility/releases).
2. Download `power-settings-visibility.exe`.
3. Run it and accept the UAC prompt.

Or build from source (below).

## Build

```powershell
git clone https://github.com/OWNER/power-settings-visibility.git
cd power-settings-visibility
cargo build --release
```

Binary:

```text
target\release\power-settings-visibility.exe
```

Helper script:

```powershell
.\build.ps1
```

UAC elevation is configured via [`.cargo/config.toml`](.cargo/config.toml)  
(`/MANIFESTUAC:level='requireAdministrator'`). [`app.manifest`](app.manifest) is a reference copy.

## Usage

1. Launch the `.exe` and approve UAC.
2. Select a setting in the left tree (not a group).
3. Read the description on the right.
4. Choose **Вкл / Выкл / Авто** — the change is written to the registry immediately.
5. Close and reopen the Windows **Advanced power settings** dialog to see the result.

## Catalog (translations)

| File | Role |
|------|------|
| [`src/power/catalog.rs`](src/power/catalog.rs) | ~154 settings (generated) |
| [`src/power/catalog_subgroups.rs`](src/power/catalog_subgroups.rs) | Subgroup titles |
| [`scripts/generate_catalog.py`](scripts/generate_catalog.py) | Generator |
| [`settings_export.csv`](settings_export.csv) | GUID export source |

Regenerate after CSV or manual text changes:

```powershell
python scripts\generate_catalog.py
```

## Safety

- Prefer a system restore point or registry export before bulk edits.
- The app mainly sets or deletes the `Attributes` DWORD under existing PowerSettings keys.
- AC/DC timeout and policy **values** are left untouched.

See [SECURITY.md](SECURITY.md).

## Project layout

```text
src/
  main.rs              Entry + elevation
  power/               Scan, visibility, RU catalog
  win/                 Win32 FFI, UI, registry, powrprof
scripts/               Catalog generator
.cargo/config.toml     Linker UAC flag
.github/workflows/     CI and release
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). PRs welcome — please keep the **no external crates** policy.

## License

[MIT](LICENSE)

---

## По-русски

Утилита для **Windows 10/11**: показывает или скрывает пункты в  
**Параметры электропитания → Настройка схемы → Изменить дополнительные параметры**.

- Селектор **Вкл / Выкл / Авто** — только видимость, не значения «От сети» / «От батареи»
- Описания на русском, блоки с жирными заголовками
- Запуск от администратора (UAC)
- Без зависимостей с crates.io

Сборка: `cargo build --release` → `target\release\power-settings-visibility.exe`
