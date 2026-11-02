# Security

## Reporting a vulnerability

Open an issue on the GitHub repository (SeanShaffer88/DevTextTools) marked as a security report, or contact the maintainer through the repository if the issue is sensitive. There is no bug bounty; fixes are credited in the changelog.

## The program's security posture

- **No network, ever.** DevTextTools contains no networking code: no update checks, no telemetry, no crash reporters, no licence servers. Text you paste — including passwords being checked or data being hashed — never leaves the machine.
- **Untrusted input is the normal case.** Pasted/typed text and file contents are treated as untrusted throughout: tools return `Result` with plain-words errors instead of panicking, file reads are size-capped in batch and replace operations, and number sorting uses a total ordering so inputs like the literal `NaN` cannot crash a tool.
- **Writes are guarded.** The program's own files (settings, session) are written atomically (temp file + flush + rename). Project Replace — the one feature that overwrites *your* files — writes a `<name>.bak` backup of each original first and writes via temp + rename, and it only ever touches files its preview listed.
- **Single instance.** A named-mutex guard prevents two windows from racing the same settings/session files; the purge commands refuse to run while a window is open.
- **Data on disk is plain text.** Settings and session files are unencrypted tab-separated text in the `data` folder beside the executable — that includes your run history and work-tab text. Anyone with access to that folder can read them; delete the folder (or run `DevTextTools --clean`) to remove them. There is no copy anywhere else.
- **Unsigned binaries.** Releases are not code-signed; verify downloads against the release's `SHA256SUMS.txt` (see [INSTALL.md](INSTALL.md)).
- **Honest crypto labelling.** MD5 and SHA-1 are labelled in-app as checksums, not safe for passwords or new security uses. The JWT decoder states that decoding is not verification; the JWT builder signs with HS256 locally. The password generator uses a time-seeded built-in RNG, and the app itself recommends a password manager's generator for high-stakes secrets.
