# Compatibility

## Operating system

- **Windows 10 or Windows 11, 64-bit (x64)** — the only supported platform for the shipped binaries.
- Releases are x64 only. There are no x86 or ARM64 builds (dropped before v0.7.2's x64-only rule; the conformance release matrix keeps it that way).

## Runtime requirements

- **None.** The executable links the compiler runtime statically: its DLL imports are Windows system DLLs only (kernel32, user32, gdi32, ole32, opengl32, …) plus `msvcrt.dll`, which is part of the Windows operating system itself. No .NET, no Visual C++ redistributable, no Java, no Electron.
- No internet connection, account, or licence server — at install time or ever.
- The installer is per-user and needs **no administrator rights**. The portable build needs no install at all.
- The binaries are **unsigned** (no code-signing certificate), so Windows SmartScreen may warn on first run — choose *More info → Run anyway* if you trust the build, and verify the download against the release's `SHA256SUMS.txt`.

## Data compatibility

- **Settings format**: versioned (`schema_version=1` since v0.8.0). Files from v0.7.2 and earlier (no version line) load unchanged and are upgraded on the next save; a file written by a newer version loads as defaults with a warning and is never overwritten by an older build. See [MIGRATION.md](MIGRATION.md).
- **Data location**: v0.8.0+ keeps everything in a `data` folder beside the executable; earlier versions used `%LOCALAPPDATA%\DevTextTools`. The first v0.8.0 run copies the old file across and leaves the original in place.
- The settings/session files are plain text (tab-separated lines) and can be inspected or backed up with any text editor.

## Building from source

- Rust 1.80+ (the crate's `rust-version`).
- The Windows build targets `x86_64-pc-windows-gnu` and needs the GCC MinGW-w64 toolchain (on the GitHub `windows-latest` runners it is preinstalled; on Linux, the apt packages `gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 nsis` — the clang-based llvm-mingw cannot link Rust windows-gnu targets).
- Development and the full test gate also run on Linux (the GUI shell compiles there; file dialogs and other Windows extras are Windows-build features).
