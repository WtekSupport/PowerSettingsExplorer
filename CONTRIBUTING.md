# Contributing

Thanks for helping improve **Power Settings Explorer**.

## Development

Requirements:

- Windows 10/11 x64
- [Rust stable (MSVC)](https://www.rust-lang.org/tools/install)
- Python 3 (only if regenerating the catalog)

```powershell
cargo build --release
.\target\release\power-settings-explorer.exe
```

Or:

```powershell
.\build.ps1
```

## Catalog (Russian descriptions)

1. Update `settings_export.csv` if GUID list changed.
2. Optionally edit `scripts/catalog_manual_ru.py` for hand-written entries.
3. Regenerate:

```powershell
python scripts\generate_catalog.py
```

4. Commit both the script changes and `src/power/catalog.rs`.

## Pull requests

- Keep the dependency policy: **no crates.io crates** — only `std` + Win32 FFI.
- Prefer small, focused PRs (UI, visibility logic, catalog, docs).
- Describe what you tested on (Windows build, admin/non-admin).
- Do **not** add Cursor / AI co-author trailers (`Co-authored-by: Cursor`, `Made-with: Cursor`, etc.). CI rejects them.

## Code style

- Match existing module layout (`src/win`, `src/power`).
- Avoid drive-by refactors unrelated to the change.
