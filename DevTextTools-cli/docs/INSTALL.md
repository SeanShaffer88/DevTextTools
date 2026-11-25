# Install

Every release ships exactly three files:

- `DevTextTools-v{VERSION}-x64-Setup.exe` — the installer
- `DevTextTools-v{VERSION}-x64-Portable.zip` — the portable build
- `SHA256SUMS.txt` — SHA-256 checksums for the two downloads above

Windows 11 x64. No administrator rights, no internet, no runtime to install first (see [COMPATIBILITY.md](COMPATIBILITY.md)).

## Portable (recommended)

1. Download the portable `.zip` and `SHA256SUMS.txt`.
2. (Optional) Verify the zip: `Get-FileHash DevTextTools-v{VERSION}-x64-Portable.zip` in PowerShell and compare with the line in `SHA256SUMS.txt`.
3. Unzip anywhere and run `DevTextTools.exe`. The folder also contains `devtexttools-cli.exe` for scripts and local AI.

The program keeps its settings and your saved work session in a `data` folder it creates beside the exe. To remove DevTextTools completely, delete the folder — or run `DevTextTools --uninstall` first to clear just the program's data files.

## Installer

1. Download the setup `.exe` (verify it the same way, if you like).
2. Run it. It installs both `DevTextTools.exe` and `devtexttools-cli.exe` per-user (into `%LOCALAPPDATA%\Programs\DevTextTools`), creates Start Menu and desktop shortcuts, and registers an uninstaller — no admin prompt.
3. Uninstall from Windows Settings → Apps, or run the installer's `Uninstall.exe`. Uninstalling also removes the program's `data` folder inside the install directory.

The binaries are unsigned, so Windows may show a "Windows protected your PC" prompt on first run — choose *More info → Run anyway* if you trust the build.

## Upgrading

Install or unzip the new version over the old one. Your settings carry a format version and are migrated automatically; the first run of v0.8.0 also moves settings from the old per-user location into the `data` folder (the original file is left untouched as a backup). Details: [MIGRATION.md](MIGRATION.md).

## Command-line maintenance

```
devtexttools-cli.exe --version            print the version
devtexttools-cli.exe --clean              remove the program's own files from its data folder (asks first)
devtexttools-cli.exe --clean --dry-run    preview what would be removed
devtexttools-cli.exe --clean --silent     remove without asking
devtexttools-cli.exe --uninstall          like --clean, plus how to remove the program files
```

Both purge commands refuse while a DevTextTools window is open, and only ever touch the program's own files (`settings.txt`, `session.txt`) in its data folder.

## Local AI and scripts

`devtexttools-cli.exe --list --json` returns all tool IDs and descriptions. Send one request as JSON on stdin with `devtexttools-cli.exe --request-json`, for example `{"tool":"case-upper","input":"hello","param":""}`. It returns one JSON response on stdout. Requests are processed locally and are not saved; the installer does not change `PATH`.
