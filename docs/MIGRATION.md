# Migration

## v0.8.0 — data folder move + settings schema v1

v0.8.0 changes where DevTextTools keeps its files and stamps a format version into them. Both changes happen automatically on the first v0.8.0 run; nothing needs to be done by hand, and nothing is deleted.

### Where the files live now

| Before v0.8.0 | v0.8.0 and later |
|---|---|
| `%LOCALAPPDATA%\DevTextTools\settings.txt` | `data\settings.txt` beside `DevTextTools.exe` |
| (work tabs/view were not saved) | `data\session.txt` beside `DevTextTools.exe` |

### How the move works

On startup the program resolves its data folder in this order:

1. **`DEVTEXTTOOLS_DATA_DIR`** — if this environment variable is set to a non-empty path, that folder is used as-is (for tests, wrappers and managed deployments). No migration runs into or out of an override folder.
2. **`data\` beside the executable** — used when it already holds any of the program's files, or when the old settings file has just been copied into it (the migration), or when it can simply be created (a fresh install).
3. **The legacy folder** (`%LOCALAPPDATA%\DevTextTools`) — used when `data\` cannot be created or the migration copy fails, e.g. the program sits in a read-only folder. An existing user never loses their files to a failed move.
4. Beside the executable, then the current folder, for exotic setups.

The migration is **one-way and copy-only**: the legacy `settings.txt` is copied into `data\`; the original is left exactly as it was, as the backup of the move. (Only `settings.txt` ever existed in the old location; `session.txt` is new in v0.8.0.)

### Settings schema version

- `settings.txt` now begins with a `schema_version=1` line.
- A file **without** the line is version 0 (every release up to v0.7.2). It loads exactly as before — the tolerant line parser is unchanged — and is stamped version 1 on the next save. This migration has a fixture test (`settings_v0_without_schema_line_loads_and_reports_v0`).
- A file with a **higher** version than the running build understands was written by a newer DevTextTools: the build starts with default settings, shows a warning in the status line, and **never overwrites the file**. Re-running the newer build reads it normally.
- The session file (`session.txt`) carries its own `schema_version=1`; a corrupt, empty, or newer-format session is ignored and the program starts fresh — never a crash.

### Window position

The last window position and size are stored in the settings file (`win` line) and restored on launch. Out-of-range or corrupt values are ignored and the default 1180×760 window opens instead.

## Earlier versions

No earlier migrations exist: up to v0.7.2 the settings file had no version stamp and lived only in the per-user location described above.
