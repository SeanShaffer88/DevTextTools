# Audit

What checks exist for this project, and the honest state of each. Anything not actually run is labelled **Unverified** rather than scored up. The v0.1.0 evidence scorecard lives in [QUALITY.md](../QUALITY.md); per-site tool coverage is audited separately in [COVERAGE.md](COVERAGE.md).

## The gate (runs in CI on every push and before any release)

- `cargo fmt --all -- --check` — formatting
- `cargo clippy --all-targets --locked -- -D warnings` — lints at zero warnings, all targets
- `cargo test --locked` — the full test suite:
  - registry checks (every tool has a unique ID; the registry holds 238 tools)
  - known-answer tests (RFC/FIPS hash vectors, Roman numerals, base conversion, subnet math, JSON/CSV round trips, Morse, edit distance, a full PDF build → extract → merge → split round trip)
  - error-path tests (bad input must produce a plain-words error, never a panic or a fake answer)
  - conformance tests: settings schema fixtures (v0 file loads as v0, future-version file is never overwritten), session save/restore round trip, purge planner, atomic writes, and the CLI surface (`--version`, `--json`, `--clean`/`--uninstall`) run end-to-end against the built binary
- Tier lint: `.project-tier` must exist and match the classification recorded in [AGENTS.md](../AGENTS.md)

## Conformance audit (v0.8.0, engineering contract v4.0, tier t3)

Checked and fixed in the v0.8.0 pass: classification record (`.project-tier` + repo AGENTS.md), static runtime (verified by DLL-import inspection of the release exe — system DLLs only), release profile (size-optimised, `panic = "abort"`), portable `data/` folder with one-way migration, `schema_version` on persistent files with tested migrations, window geometry + work-session persistence, atomic saves, Project Replace backups, single-instance mutex, full CLI maintenance surface (`--version` / `--clean` / `--uninstall` / `--json`), and the release pipeline (gated CI builds, versioned asset names, `SHA256SUMS.txt`).

## Standing checks by hand

- Release assets are re-downloaded after publishing and their SHA-256 compared against `SHA256SUMS.txt` before a release is called verified.
- README screenshots are real captures from a Windows machine only — never mockups or placeholders.

## Unverified

- **Windows runtime behaviour of each new build** until it has actually been run on a Windows PC (window layout, single-instance behaviour, installer, session restore). CI compiles and tests the code; it does not click through the window.
- **Idle RAM** against the tier's 40 MB guideline — not yet measured on a release build.
- **Performance budgets** in [PROJECT_RULES.md](../PROJECT_RULES.md) (start < 2 s, ops < 100 ms on 1 MB input) are design budgets, not measurements.
- **Accessibility** beyond egui/accesskit defaults — no dedicated screen-reader pass has been run.
