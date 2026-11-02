# Architecture

DevTextTools is a single Rust crate with a strict split: **everything a tool does lives in the library; the binary is only a shell** (window + command line). Project-level rules are in [PROJECT_RULES.md](../PROJECT_RULES.md) (Elite rule sheet) and [DECISIONS.md](../DECISIONS.md) (decision log); this file describes the code as it stands.

## Layout

| Path | What it holds |
|---|---|
| `src/lib.rs` | Library root: the operation registry re-exports, the shared `Rng`, `split_param`, number parsing helpers |
| `src/ops/` | The 238 tools, one module per family (`text`, `encode`, `dev`, `devpack`, `devpack2`, `calc`, `convert`, `morecalc`, `data2`, `hashgen`, `fancy`, `files`); `src/ops/mod.rs` is the registry |
| `src/pdf.rs` | PDF operations (info, text extract, merge, split) on `lopdf` |
| `src/paths.rs` | Where the program's files live: portable `data/` resolution, the one-way legacy migration, atomic writes, the purge list |
| `src/win32.rs` | The single-instance mutex (raw `CreateMutexW` FFI, no `windows` crate) |
| `src/main.rs` | The binary shell: egui window (six views, work tabs, History rail), the CLI dispatch, settings/session load & save |
| `tests/ops_tests.rs` | Registry + known-answer + error-path integration tests |
| `tests/conformance.rs` | v0.8.0 conformance tests: data-folder mechanics and the CLI maintenance surface, end-to-end against the real binary |

## The one dispatch rule

Every tool is an *operation*: a pure function `(input: &str, param: &str) -> Result<String, String>` registered once in `src/ops/mod.rs`. The window, the CLI and the tests all call the same `run_op(id, input, param)`. Multi-value parameters use one convention everywhere: `|||` as the separator. Because of this, a tool cannot exist in the window but not on the command line.

## Data flow & persistence

- Settings (favorites, recents, saved pipelines, snippets, saved regex patterns, auto-run flag, last-20-run history) and the work session (work tabs, active tab, current view) are plain-text, tab-separated files in the `data/` folder beside the executable — resolution order and migration in `src/paths.rs` and [MIGRATION.md](MIGRATION.md).
- Every write of the program's own files is atomic: temp file in the same folder → `sync_all` → rename over the target (`paths::atomic_write`). The session is autosaved a short debounce after it changes and again on exit; settings save whenever they change.
- Files the program writes into *your* folders (Project Replace) are guarded the other way: a `<name>.bak` backup of the original is written first, then the same temp + rename write.
- Both persistent formats carry a `schema_version`; loaders migrate older files in memory and refuse to overwrite files from a newer version (see `parse_settings` / `parse_session` in `src/main.rs`).

## Single instance

Window launches claim the named mutex `Global\DevTextTools-{86D97EB5-3EF9-4C34-8578-5B4FFB955136}` for the life of the process (`src/win32.rs`); a second launch prints "already running" and exits. CLI runs never claim it — `--clean`/`--uninstall` only probe it and refuse while a window is open, so a purge can never delete files out from under a running window.

## Rendering & platform

egui/eframe 0.33 (glow), immediate mode, hand-rolled layout with measured button rows (labels can never be cut off — see the v0.7.2 changelog entry). The Windows build targets `x86_64-pc-windows-gnu` with the GCC MinGW toolchain and a static compiler runtime (`.cargo/config.toml`); `build.rs` embeds the exe icon via `windres` when available. No network code exists anywhere in the crate.
