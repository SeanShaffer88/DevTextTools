# Changelog

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
