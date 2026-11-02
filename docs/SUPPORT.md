# Support

## Getting help

- **Bugs and feature requests**: open an issue on the GitHub repository (SeanShaffer88/DevTextTools). Include the version (`DevTextTools --version`), what you ran (window steps or the exact command line), the input that caused it if you can share it, and what happened instead.
- **Questions about a tool**: every tool's expected input is shown in the window under the Input box, and the full tool list is one command away: `DevTextTools --list` (or `--search <word>` to filter).

## Before you file

- Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md) — the common cases (SmartScreen warning, "already running", settings not restored, undoing a Project Replace) are answered there.
- Per-site coverage and known gaps are listed honestly in [COVERAGE.md](COVERAGE.md) and [ROADMAP.md](ROADMAP.md) — if a tool from the source sites is missing, it is probably recorded there.

## What support can and can't do

DevTextTools is free, offline software with no accounts and no telemetry: there is no server-side data to look up, reset, or recover. Everything the program saves is in the `data` folder beside the executable (see [MIGRATION.md](MIGRATION.md)), so include a copy of `settings.txt` in a bug report only if you're comfortable sharing your favorites/pipelines/history — it is never required.
