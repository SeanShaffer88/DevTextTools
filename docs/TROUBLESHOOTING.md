# Troubleshooting

## Windows says "Windows protected your PC"

The binaries are unsigned (there is no code-signing certificate). Click **More info → Run anyway** if you trust the build. You can verify the download first: compare its SHA-256 (`Get-FileHash <file>` in PowerShell) with the release's `SHA256SUMS.txt`.

## The program says "DevTextTools is already running"

Only one window can run against a data folder. Find the existing window (it may be behind others or on another virtual desktop) and close it if you really want a fresh launch. The same guard makes `--clean` / `--uninstall` refuse while a window is open — close the window, then run the command again.

## My settings / work didn't come back

- Settings and the work session live in the **`data` folder beside `DevTextTools.exe`** (v0.8.0+). If you moved the exe without its `data` folder, the program starts fresh and makes a new one. Move the folder with it.
- If the exe sits somewhere it cannot write (e.g. `C:\Program Files` without rights), the program falls back to the legacy per-user folder (`%LOCALAPPDATA%\DevTextTools`). Move the portable build somewhere writable to keep everything in `data`.
- A warning in the status line — "settings file was written by a newer DevTextTools" — means this build is older than the one that saved the file. Your file is untouched; run the newer build.
- A corrupt session file is ignored on purpose (fresh start, no crash). Settings load tolerantly line by line, so one bad line never loses the rest.

## I ran Project Replace on the wrong files

Every file Apply overwrote first had its original written beside it as **`<name>.bak`**. Rename the `.bak` file back over the changed file to restore it.

## A tool gives an error instead of an answer

That's by design: tools return plain-words errors (bad number, missing parameter, malformed JSON…) rather than a wrong answer. Check the parameter format shown under the Input box — multi-value parameters are separated with `|||` (for example `10|||16`).

## `--clean` says there is nothing to remove

The data folder in use holds neither `settings.txt` nor `session.txt`. You may be looking at a different copy of the program — the command prints the data folder it resolved. The `DEVTEXTTOOLS_DATA_DIR` environment variable, when set, overrides the folder.

## Building from source fails to link (windows-gnu)

Use the real GCC MinGW toolchain (`gcc-mingw-w64-x86-64` / `g++-mingw-w64-x86-64` apt packages on Linux; preinstalled on GitHub's `windows-latest`). The clang-based llvm-mingw cannot link Rust `x86_64-pc-windows-gnu` targets (no `libgcc`/`libgcc_eh`). NSIS (`makensis`) is needed only for the installer, and `windres` only for the exe icon — the build skips the icon gracefully without it.
