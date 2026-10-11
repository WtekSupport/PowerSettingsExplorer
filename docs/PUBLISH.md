# Publishing

Repository: https://github.com/WtekSupport/PowerSettingsExplorer

## Push / release

```powershell
cd d:\Develop\power-settings-visibility

git push -u origin main
git tag v0.1.0
git push origin v0.1.0
```

The [Release workflow](../.github/workflows/release.yml) builds the Windows `exe` and attaches it to the GitHub Release for tag `v*`.

## Local checks

```powershell
powershell -File .\scripts\check-no-ai-authors.ps1
cargo build --release
```

Install local commit-msg hook (strips AI trailers):

```powershell
Copy-Item .\scripts\git-hooks\commit-msg .\.git\hooks\commit-msg -Force
```
