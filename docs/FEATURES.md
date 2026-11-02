# Features

DevTextTools is one offline Windows program holding **238 tools**, each usable in the window and on the command line through the same operation registry — there are no window-only tools.

## The tools

Grouped in ten categories:

- **Case** — UPPERCASE, lowercase, Title, Sentence, camelCase, PascalCase, snake_case, kebab-case, CONSTANT_CASE, tOGGLE, aLtErNaTiNg
- **Lines** — sort (A→Z, Z→A, by length, numeric), reverse, shuffle, dedupe (incl. ignore-case), remove empty lines, keep/remove lines containing…, prefix/suffix, line numbers, trim, wrap/unwrap, word sort/dedupe, line-ending tools
- **Transform** — reverse text/words, repeat, strip spaces/punctuation/numbers/vowels/accents/HTML, slugify, ROT13, Caesar, Pig Latin, upside-down, leet, Morse, NATO alphabet, fancy Unicode styles
- **Analyse** — counts + reading time, word & letter frequency, find & replace, grep, extract emails/URLs/numbers, text diff + unified-diff export, edit distance, log analyzer, stack trace cleaner
- **Encode / Decode** — Base64 (+URL-safe), Base32, hex, binary, URL, HTML entities, Unicode escapes, code points, decimal bytes, quoted-printable, string escape/unescape, mojibake fixer, invisible character revealer
- **Hash / Crypto** — MD5, SHA-1, SHA-256, SHA-384, SHA-512, all-at-once, CRC32, HMAC-SHA256 (MD5/SHA-1 labelled in-app as checksums only)
- **Developer / Format** — JSON format/minify/validate/stats and conversions (CSV, YAML, query string, TOML, TypeScript types), XML format, SQL format (basic), CSS minify (basic), JWT decoder and JWT builder/signer (HS256), regex tester + explainer, cron explainer, URL parser, .env tools, UUID validator, JSONPath queries, JSON Schema check (basic), curl → code, JSON structural diff, mock-data generator, .gitignore generator, commit-message checker, changelog maker
- **Generators** — UUID/GUID, password (+ strength check), random string/number/color, Lorem Ipsum, dice, coin flip, random picker
- **Converters** — number bases 2–36, Roman numerals, temperature, length, weight, area, volume, speed, time, data storage, pressure, energy, angle, Unix timestamp ↔ date, bytes, number → words, decimal → fraction, colour HEX↔RGB/HSL + palette tools, aspect ratio
- **Calculators** — expression calculator, percentage + % change, statistics, average/median/sum, factorial, Fibonacci, GCD/LCM, primes, tip & split, BMI, age/date difference, IPv4 subnet, discount, loan/mortgage, compound interest
- **Files & PDFs** — file hash/info, file ↔ Base64, PDF info, PDF text extraction, PDF merge and split

Per-site coverage and honest gaps: [COVERAGE.md](COVERAGE.md).

## The workspace

- **Six views** — Tools, Timers (stopwatch, countdown, Pomodoro), Pipelines (chain tools; saved chains persist), Regex Lab (live matches, saved patterns), Replace (project find & replace with preview), JSON Tree (fold-out tree with copyable JSONPaths)
- **Work tabs** — several jobs side by side, each with its own tool and text; the session is saved as you go and restored on the next launch, including after a crash
- **★ Favorites / ◷ Recent** categories, **Ctrl+K** quick launcher, search box, **snippet library**, **smart paste** (recognises JSON/JWTs/hashes/stack traces), opt-in **auto-run on paste**, **batch runs** over many files, drag & drop (a dropped PDF shows its info), **History rail** (last 20 runs persist; RE-OPEN restores a run)
- **Undo/Redo** for whole actions; **Output → Input** chaining
- The window remembers its position and size; a single-instance guard keeps one window per data folder

## Command line

Every tool runs headless: `DevTextTools <tool-id> [--param X]` reads stdin and writes stdout, plus `--list`, `--search`, `--version`, `--json` output, `pdf-merge` / `pdf-split`, and the maintenance commands `--clean` / `--uninstall` (see [INSTALL.md](INSTALL.md) and the README's Command line section).

## Data & safety properties

- Fully offline — no network calls at all, no accounts, no telemetry
- All program files live in one `data` folder beside the executable (settings carry a `schema_version`); saves are atomic (temp file + flush + rename)
- Project Replace writes a `<name>.bak` backup before overwriting any file
- Free (MIT), no ads, no in-app purchases; optional donations only
