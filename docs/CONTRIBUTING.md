# Contributing

Thanks for helping. DevTextTools is a single Rust crate; the layout and the one hard rule are described in [ARCHITECTURE.md](ARCHITECTURE.md).

## The one hard rule

Every tool is an *operation* — a pure function `(input, param) -> Result<output>` registered once in `src/ops/mod.rs`. The window, the CLI and the tests all dispatch through that registry. **A feature that only exists in the window is not acceptable**: if it can be clicked, it must be runnable from the command line through the same registry entry.

## Adding a tool

1. Write the operation in the matching `src/ops/` module (input and parameter are untrusted — return plain-words `Err` for bad input, never panic; multi-value parameters split on `|||`).
2. Register it with a unique kebab-case ID in that module's `ops()` list.
3. Add known-answer tests (and at least one error-path test) in `tests/ops_tests.rs`.

## Before you open a pull request

Run the full gate — all three, in full:

```
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

CI runs exactly this (plus the tier lint) and must be green. Keep changes surgical: complete files when new, small diffs when editing, and update the docs your change touches (README / docs / CHANGELOG "Unreleased" section) in the same change.

## House rules

- No network code, no telemetry, no new dependency without a stated reason (the crate is deliberately small: egui/eframe, lopdf, regex, base64, serde_json and the hash digests).
- No winget packaging. Releases are cut by the maintainer from tags; don't add release automation beyond `.github/workflows/`.
- Public docs state product facts only.
