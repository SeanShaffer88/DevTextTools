# Decisions
- 2026-10-25: Name = DevTextTools. Stack = Rust + egui/eframe (matches ConvertDesk pattern; cached crates; cross-builds to windows-gnu). Dark black/red family theme.
- 2026-10-25: Tool interface = (input text, param string) with `|||` as the one multi-value separator — one consistent pattern to learn, works identically in GUI and CLI.
- 2026-10-25: Unit converters are parametric (one tool per quantity, param = from|||to) instead of one tool per pair — this is how 291 alotoftools "tools" become 10 DevTextTools tools with identical capability.
- 2026-10-25: Media/PDF/per-language formatters deferred (docs/COVERAGE.md gives the reasons per site); web-dependent tools out of scope permanently (offline by design).
- 2026-10-25: RNG = small built-in xorshift seeded from system time (no extra dependency); password tool labels its own limit in-app.

## Rename + v0.2/v0.3 (SEAN, 2026-10-25)
- SEAN renamed the program **DevTextTools** and directed: build the v0.2 breadth slice, then the v0.3 files+PDF slice, **commit locally but do NOT push** (no GitHub publish yet).
