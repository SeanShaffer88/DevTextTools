# Coverage — what DevTextTools takes from each of the 10 sites

Researched 2026-10-25 by fetching each site's home / tools index page.
Counts below are what was *verified from page text*, not the sites' own
marketing claims (noted where they differ).

| Site | Verified on its pages | DevTextTools v0.1.0 coverage |
|---|---|---|
| onlinetexttools.com | ~106 text tools | Case, lines, transform, encode family (Base64/32, hex, binary, Morse, ROT/Caesar), counts, find/replace, extract — covered |
| computerhope.com/text-tool | 1 page, ~40 options (remove/replace classes of characters/lines) | Covered via Lines + Transform + Find & Replace tools |
| dev-toolbox.tech | Claims 312+; only JSON, UUID, regex, subnet were text-verifiable (JS-rendered) | All 4 verifiable capabilities covered, plus the standard dev-toolbox set (JWT, cron, URL parse, query↔JSON) |
| formatter.org | ~385 tools in 28 categories (claims 200+) | JSON/XML/SQL/CSS formatting + data conversion covered in v0.1; per-language code formatters (C/Java/Python/…) are **deferred** — they need real parsers per language, see ROADMAP |
| alotoftools.com | 969 tools enumerated (claims ~1,027): Time & Date 181, Calculators & Math 243, Finance 22, Unit Converters 291, Text 111, Random 29, Health 26, Games 13, Education 45 (+ more categories on homepage) | Unit converters are *parametric* in DevTextTools: one Length tool covers every length pair (the site lists each pair as a separate "tool"), same for weight/area/volume/speed/time/data/pressure/energy/angle/temperature. Core calculators (expression, %, stats, factorial, Fibonacci, GCD/LCM, primes, tip, BMI, loan, compound, discount, subnet, date difference) covered. The long tail of single-formula calculators (geometry/physics/finance variants) is deferred — see ROADMAP |
| pinetools.com | ~190 listings (~140 unique; 108 are image tools incl. bulk duplicates) | Text & lists, numbers, date & time, randomness, programming covered. **Image tools are NOT in DevTextTools v0.1** — SEAN already has ConvertDesk for images; a future slice can add basic image ops or link out. Video tools likewise deferred |
| onlineminitools.com | Claims 2,300+; only 24 tools named on homepage (+7 via search) | All homepage-named text/dev tools covered (JSON, unicode, character frequency, design-token and Avro/HAR niche tools deferred) |
| textmachine.org | Claims 140+ (FAQ says 60+ — contradictory); 0 tools individually listed; 9 tool pages verified via search | Word counter, smart quotes (via find/replace), mailto (URL tools), cipher solvers — basic ciphers covered (Caesar/ROT13/Morse); Vigenère/substitution *solvers* deferred |
| texttools.org | 52 tools, all on homepage | Covered: manipulation, cleaner, formatting, unicode/fancy text (partial — upside-down + leet in v0.1), encode/decode, random generators. Web-scraping tools are **out of scope** (they need the live web; DevTextTools is offline by design) |
| mcstumble.com/tools | 77 tools: Text 9, Image/Video/Audio 23, Productivity 14, Developer 18, Design 5, PDF 8 | Text + Developer covered. Image/video/audio and PDF tools deferred (ConvertDesk covers media conversion; PDF is a future slice). Productivity timers/pomodoro deferred to a GUI-widgets slice |

## Honest limits of v0.1.0

- **Not literally all ~2,000+ listed "tools".** A large share of the sites'
  counts are (a) the same tool listed once per input/output pair
  (unit converters, image bulk variants), and (b) single-formula
  calculators. DevTextTools v0.1 covers the *capabilities* behind the great
  majority of listings for text, developer, encoding, hashing, generating,
  converting and everyday calculating — with the parametric design above.
- Media (image/video/audio editing), PDF editing, and per-language code
  formatters are real gaps, listed in `docs/ROADMAP.md`, not hidden.
- Everything runs offline. Anything on those sites that inherently needs
  the internet (web scraping, downloader tools, "my IP") is out of scope
  by design and is never simulated with fake results.
