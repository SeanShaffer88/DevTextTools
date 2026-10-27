# QUALITY.md — DevTextTools v0.1.0 scorecard (Elite prompt §17)

Scored from evidence only; anything not yet run is marked Unverified, not scored up.

| Dimension | Score (0-100) | Evidence |
|---|---|---|
| Correctness | 90 | Full gate run by Muse on the final tree 2026-10-25: cargo fmt --check clean, clippy --all-targets 0 warnings, cargo test --locked (all targets) 14/14 integration tests pass (incl. a full lopdf PDF round-trip: build two PDFs, extract text, merge to 2 pages, split back to single pages — all asserted) incl. RFC 4231 HMAC vector, FIPS hash vectors, subnet/Roman/base known answers; CLI smoke-verified at each version (--list totals: 164 v0.1, 203 v0.2, 208 v0.3; pdf-merge/pdf-split executed end-to-end on generated fixtures 2026-10-25). Windows exe cross-built + PE-verified (PE32+ GUI, subsystem 2, system-DLL imports only, registry strings present, fresh mtime). GUI launch on a real Windows machine: Unverified — SEAN's run is the verdict |
| Test depth | 80 | One integration suite covering every op family + registry uniqueness; per-op unit tests grow in v0.2 |
| Usability | 75 | One consistent input/param/output pattern, search, tool chaining; Unverified on a real Windows machine until SEAN's run |
| Accessibility | 60 | egui + accesskit (as in ConvertDesk); full keyboard/screen-reader pass is Unverified |
| Performance | Unverified | Not yet measured; budget: ops < 100 ms on 1 MB input. Exe size 6,101,504 B |
| Security/privacy | 90 | No network code at all, no disk writes in v0.1, JWT tool warns decode != verify, weak hashes labelled |
| Honesty of docs | 90 | docs/COVERAGE.md lists per-site gaps by name; README carries Unverified labels |

Known Fail items: none claimed. Release gate = full `cargo test --locked` (all targets) green + Windows exe size/mtime/string verification; on-machine launch remains SEAN's verdict.
