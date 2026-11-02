# Changelog

## [0.8.0] — Unreleased
Engineering-contract conformance release (tier t3). No tool behaviour changes except the two fixes noted; the work is where the program keeps its files, crash safety, and how it is built and released.

- **Your files moved next to the program.** Settings and the new saved work session now live in a `data` folder beside `DevTextTools.exe` instead of `%LOCALAPPDATA%\DevTextTools`. On first run the old settings file is copied across (the original is left untouched); if the `data` folder cannot be written, the old location is used instead, so nothing is lost to a failed move. See [MIGRATION.md](MIGRATION.md). `DEVTEXTTOOLS_DATA_DIR` can point the data folder elsewhere.
- **Your work is saved as you go.** The work tabs (tool, input, parameter, output), the active tab and the current view are written to `data\session.txt` whenever they change and on exit, and restored on the next launch — a crash or power cut no longer loses what you were doing. The window also reopens at its last position and size.
- **Settings carry a format version.** `settings.txt` now starts with `schema_version=1`. Older files load unchanged and are upgraded on the next save; a file written by a newer DevTextTools loads as defaults with a warning instead of being overwritten.
- **Saves are crash-safe.** Settings and session are written to a temporary file, flushed, and renamed over the old file.
- **Project Replace keeps a backup.** Before Apply overwrites a file it writes a `<name>.bak` copy beside the original, and the write itself is temp-file-then-rename.
- **New maintenance commands.** `DevTextTools --version` prints the version. `--clean` removes the program's own files from the data folder (with `--dry-run` to preview and `--silent` to skip the confirmation); `--uninstall` does the same and explains how the program files are removed. `--json` prints a tool run's result — or a version/clean report — as one JSON object. Both purge commands refuse while a DevTextTools window is open.
- **One window at a time.** A second launch now prints "already running" and exits instead of opening a second window over the same files.
- **Bug fix.** The statistics and numeric-sort tools could crash when the input contained the literal `NaN`; number sorting now uses a total ordering and never panics.
- **Releases are x64 only, in one shape.** Each release now ships `DevTextTools-v{version}-x64-Setup.exe`, `DevTextTools-v{version}-x64-Portable.zip` and `SHA256SUMS.txt`. Builds are made by the new CI/release workflows, which run the full check (formatting, lints, all tests) before packaging.
- **Static runtime.** The Windows build links the compiler runtime statically — the exe imports only Windows system DLLs (plus the OS's own `msvcrt.dll`), so no separate runtime install is needed.
- **Documentation.** The changelog moved to `docs/` and the standard document set was adopted: AUDIT, FEATURES, COMPATIBILITY, ARCHITECTURE, INSTALL, MIGRATION, TROUBLESHOOTING, SUPPORT, SECURITY, CONTRIBUTING, CODE_OF_CONDUCT, MAINTAINERS, GOVERNANCE and CREDITS now live beside it. The portable note (PORTABLE.txt) now correctly says the program writes its `data` folder.


## [0.7.2] — 2026-10-27
Button rows sized by label width — no button is cut off.

- The tool ribbon, category chips, command bar and file/PDF button rows now measure every button's label and pack rows by real width: a row wraps to the next line before any button can run off the right edge, and the buttons in a row still stretch to fill the full width. (In v0.7.1 the rows were divided into equal slots; in categories with long tool names — Lines, Analyse, Developer / Format, Generators, Calculators, Fancy Text, Files — the last button in a row was cut off mid-label.)
- Work tabs show the tool's full name instead of cutting it at 18 characters; the tab row scrolls sideways when there are more tabs than fit.

## [0.7.1] — 2026-10-25
Toolbar stretching and resize safety.

- The category and view chip rows now stretch to fill the toolbar, like the tool ribbon and the command bar already did.
- The bottom command bar spans the full window width, so the History rail can no longer cover its last button.
- The window has a minimum size (1120×640): it refuses to shrink to a size where toolbars or editors would be cut off. The work-tab row scrolls sideways when there are more tabs than fit, and the snippet name field stretches to fill its row.

## [0.7.0] — 2026-10-25
Terminal design.

- New look: phosphor-green accent on near-black, monospace text, square corners throughout.
- Navigation is now a row of category chips plus view chips (Tools, Timers, Pipelines, Regex Lab, Replace, JSON Tree) across the top.
- A DISPLAY box above the editor shows what the selected tool expects; Input and Output sit side by side, each with its parameter or status line underneath.
- History rail: every successful run is recorded with its time, tool, input and result; RE-OPEN restores the tool with its input and parameter. The last 20 runs are kept between sessions.
- Tooltips on the chips, tool buttons, work tabs, command buttons and History controls.

## [0.6.1] — 2026-10-25
- The tool ribbon is capped at three rows with its own scrollbar, so tall categories no longer squeeze the input/output workspace.

## [0.6.0] — 2026-10-25
Developer enhancements and 22 new tools (238 total).

- Work tabs: several jobs side by side, each with its own tool, text and settings.
- Undo/Redo for whole actions; batch run of a tool over many files; project find & replace with a preview before applying.
- Smart paste (UUIDs, JSON, JWTs and URLs are recognised and offered an action), auto-run on paste (opt-in), a snippet library, a JSON Tree Explorer, and saved patterns in Regex Lab.
- New tools include a JWT builder/signer (HS256), mock data generator, JSON structural diff, .env compare/merge, log analyzer, stack trace cleaner, mojibake fixer and invisible character revealer.

## [0.5.0] — 2026-10-25
Developer Pack.

- ★ Favorites and ◷ Recent categories, a Ctrl+K quick launcher, and drag & drop of files onto the window.
- Pipelines view: chain tools into a saved sequence that runs as one step. Regex Lab: test patterns with live matches.
- 8 new tools (216 total).

## [0.4.2] — 2026-10-25
- App icon for the window, the executable and the installer.

## [0.4.1] — 2026-10-25
- The tab bar, ribbon and command buttons stretch to fill their bars; the search box stretches too.

## [0.4.0] — 2026-10-25
- Ribbon Studio layout: category tabs across the top and a ribbon of tool buttons under them; the command bar stays at the bottom.

## [0.3.2] — 2026-10-25
- Layout redesign: category menus in the upper panel, input/output in the middle, command buttons in a bottom panel.

## [0.3.1] — 2026-10-25
- Fixed symbol/emoji text showing as boxes (fallback fonts), Pomodoro now starts on Focus, one-click PDF Info… and PDF → Text… buttons, and a Clear button for search.

## [0.3.0] — 2026-10-25
- Files category: file hash, file info, file to/from Base64, PDF info and PDF text extraction.
- PDF merge and split, in the window and on the command line. Open/save file buttons in the window.
- 208 tools.

## [0.2.0] — 2026-10-25
- Renamed from TextForge to DevTextTools.
- 39 new tools (203 total): fancy Unicode text, Vigenère cipher, YAML/XML/CSV conversions, geometry, Ohm's law, GPA and BMR calculators, date tools.
- Timers view: stopwatch, countdown and Pomodoro.

## [0.1.0] — 2026-10-25
First release. 164 tools across 10 categories (Case, Lines, Transform, Analyse, Encode/Decode, Developer/Format, Hash/Crypto, Generators, Converters, Calculators), each available in the window and on the command line through the same operation registry. Fully offline. Coverage vs the 10 source sites and honest gaps: docs/COVERAGE.md.
