# DevTextTools — Project Rules (condensed from Adaptive Engineering Contract V4.0)

## Project Manifest
- tier: 3 (rich Windows GUI) — **by SEAN's decision, 2026-11-02**
- tier rationale: an egui/eframe immediate-mode suite with a tool ribbon (capped at three rows), category chips, six views (Tools, Timers, Pipelines, Regex Lab, Replace, JSON Tree), multiple work tabs and a History side rail in an 1180×760 window — a rich multi-pane, render-driven surface. The contract's classification companion listed this project under t1b; that listing is **superseded for this project** by SEAN's 2026-11-02 decision, following the CalcDex precedent (egui multi-pane suite = t3) and the companion's own richness test (a rich multi-pane editor or render-driven surface → t3). Note: egui is not named in the contract's §6 renderer list (Slint, Direct2D, Godot, raylib); the t3 placement follows the adopted classification companion's test plus SEAN's decision.
- targets: windows-x64, built for `x86_64-pc-windows-gnu` with the real GCC MinGW toolchain (llvm-mingw/clang cannot link Rust windows-gnu targets — see `build-072-x64.sh` preflight)
- toolchain: Rust 2021, eframe/egui 0.33 (glow renderer); static C runtime for the gnu target via `.cargo/config.toml` (`target-feature=+crt-static`) — verification outcome recorded under "Build verification" below
- renderer: egui immediate-mode GUI over glow
- release assets: DevTextTools-v{VERSION}-x64-Setup.exe + DevTextTools-v{VERSION}-x64-Portable.zip + SHA256SUMS.txt (the exe installer ships with every desktop release; no MSI, ever — SEAN's amendment)
- tier overrides: idle RAM cap 40 MB (t3 rule); LICENSE kept under that exact name at the repo root for GitHub license detection — a documented deviation from the standard docs set's LICENSE.md
- project records: PROJECT_RULES.md (Elite rule sheet), DECISIONS.md and QUALITY.md remain the project's own records and are not replaced by this file
- dropped: Sections 3, 4, 5, 7, 8 (other tiers); non-Windows CI steps
- condensed from: Adaptive Engineering Contract V4.0 (2026-11-01, with SEAN's five amendments)

## Build verification
- crt-static (slice 2, 2026-11-02): `.cargo/config.toml` sets `target-feature=+crt-static` for `x86_64-pc-windows-gnu`; the toolchain **accepts** it. Verified on this VM with the real GCC MinGW: `cargo build --release --locked --target x86_64-pc-windows-gnu` links cleanly, and `x86_64-w64-mingw32-objdump -p` on the resulting `DevTextTools.exe` shows **no compiler-runtime DLL imports** (no `libgcc_s_seh-1.dll`, no `libstdc++-6.dll`, no `libwinpthread-1.dll`) — the import list is Windows system DLLs only (kernel32, user32, gdi32, ole32, opengl32, …), identical to the shipped v0.7.2 exe. The one C-runtime import is `msvcrt.dll`, which is a Windows operating-system component present on every supported install, not a redistributable the user must install. (Rust's gnu target already links libgcc/winpthread statically by default; the flag pins that guarantee in config rather than leaving it to defaults.)

## Universal mandates (§2 — Class C, never violated silently)
- Zero cloud / zero telemetry: 100% functional air-gapped; no telemetry endpoints, license servers, crash reporters, or phone-home. DevTextTools makes no network calls at all — there is no update check.
- Respect the user: no unprompted toasts, promotional links, browser-default changes, or unapproved startup entries. Offline help in the window and via `--help`. Everything the program writes lives in one documented location (`data/` beside the executable; see docs/MIGRATION.md).
- Config schema versioning: persistent files carry a `schema_version`; loaders migrate upward explicitly and never silently overwrite or reset user data; every schema bump ships a tested migration fixture.
- Token economy: complete files when new; surgical diffs for edits; status in concise bullets.

## Tier 3 rules (§6, as they apply here)
- Static runtime linkage mandatory (crt-static for the gnu target — see Build verification); zero Electron/web runtimes.
- Idle RAM < 40 MB; UI thread < 16 ms; no blocking first frame.
- Mockup/wireframe confirmation before new top-level windows or primary layout changes.
- Every capability in the window is also usable from the command line (the registry in `src/ops/mod.rs` is the single dispatch; `src/main.rs` is GUI + CLI shell only). CLI maintenance commands: `--help`, `--version`, `--clean`, `--uninstall`, with `--dry-run`/`--silent`/`--json` modifiers.
- Single-instance: GUID-prefixed named mutex `Global\DevTextTools-{86D97EB5-3EF9-4C34-8578-5B4FFB955136}` for window launches.
- Disk mutations to the app's own files are atomic: temp file → flush (`sync_all`) → rename over the target. Folder Replace writes a `<name>.bak` backup beside each user file before overwriting it.
- `unwrap()`/`expect()` banned in hot paths and untrusted-input code (pasted/typed text and file contents are untrusted); errors surface as plain-words messages, never panics. Number sorting uses `f64::total_cmp` (user text can contain `NaN`).
- Releases (tag + GitHub Release) require explicit human confirmation; branches/PRs/commits do not. CI runs the full gate (fmt check, clippy `-D warnings`, full tests) and the tier lint before any release; asset names match the matrix exactly; checksums in one pass. No winget, ever.
