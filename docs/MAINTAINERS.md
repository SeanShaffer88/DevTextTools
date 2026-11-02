# Maintainers

- **Sean Shaffer** (GitHub: SeanShaffer88) — creator and sole maintainer: design, releases, issue triage, and final say on scope.

## Release process (summary)

1. Work lands on `main` through the gated CI (fmt + clippy `-D warnings` + full tests + tier lint).
2. A version is tagged `v{VERSION}` only with the maintainer's explicit confirmation.
3. The release workflow builds the gated tree for `x86_64-pc-windows-gnu`, packages the NSIS installer and the portable zip, writes `SHA256SUMS.txt`, and attaches all three to the GitHub release.
4. Published assets are re-downloaded and hash-checked against `SHA256SUMS.txt` before the release is called verified.

See [GOVERNANCE.md](GOVERNANCE.md) for how decisions are made.
