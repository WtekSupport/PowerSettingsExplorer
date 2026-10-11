# Publishing to GitHub

Replace `OWNER` with your GitHub username or organization everywhere it appears
(`README.md`, issue templates, `Cargo.toml`).

## One-time setup

1. Create an empty public repository named `power-settings-visibility` on GitHub  
   (no README / license — this repo already has them).
2. From this folder:

```powershell
cd d:\Develop\system\power-settings-visibility

git init -b main
git add .
git commit -m "Initial release: Power Settings Visibility 0.1.0"

git remote add origin https://github.com/OWNER/power-settings-visibility.git
git push -u origin main
```

3. On GitHub: **Settings → Pages** (optional), enable **Issues**, **Discussions** if you want.
4. Install [GitHub CLI](https://cli.github.com/) if you prefer `gh repo create`.

## Cutting a release

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The [Release workflow](../.github/workflows/release.yml) builds Windows `exe` and attaches it to the GitHub Release.

## Checklist before going public

- [ ] Replace `OWNER` in README badges and links
- [ ] Set `repository` / `homepage` in `Cargo.toml`
- [ ] Confirm MIT copyright line in `LICENSE`
- [ ] Optional: add a screenshot under `docs/screenshot.png` and link it in README
- [ ] Run `cargo build --release` locally once more
