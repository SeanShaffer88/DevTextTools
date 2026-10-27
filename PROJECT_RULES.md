# DevTextTools — Project Rules
Prompt in force: The Elite Software Master Prompt v1.1 · Annexes: A (Windows), B (Storytold, at substantial slices), C1/C2
Purpose in one line: Every tool from 10 online tool sites, in one offline Windows program.
Frozen foundation: src/lib.rs + src/ops/ modules are the tool registry; every tool is a pure operation (input, param) -> Result<output>; GUI (src/main.rs) and CLI dispatch through the same registry — no GUI-only features.
Build: cargo build --release --locked · Full gate: cargo test --locked (ALL targets) · Test corpus: tests/ops_tests.rs (grows per slice)
Budgets: single exe, starts < 2 s on a ordinary PC, every op < 100 ms on 1 MB input except where noted; no network at all, ever.
Coverage floors: Elite defaults (85% core / 70% overall / 100% safety paths) at v1.0; v0.1 relies on known-answer tests per op family.
Special rules: no winget (no signing cert); releases = loose installer exe + one portable zip; unsigned, stated honestly; MIT; README TL;DR first, real screenshots only.
Known limitations live at: docs/COVERAGE.md · Decision log: DECISIONS.md · QUALITY.md: QUALITY.md
