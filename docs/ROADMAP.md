# DevTextTools roadmap

Only items that are real, planned work — in rough priority order.

## v0.2 — breadth within the current kinds
- Fancy Unicode text styles (bold/italic/cursive/bubble/small caps/strikethrough generators from texttools.org)
- Vigenère cipher encode/decode + substitution cipher helper
- YAML → JSON, XML ↔ JSON, TOML peek (data-format family from formatter.org)
- More calculators from alotoftools: geometry set (circle/triangle/rectangle…),
  Ohm's law / voltage divider, grade/GPA, pace, concrete/material estimators —
  each is one formula, added in batches with known-answer tests
- Date tools: add/subtract days, business days, day-of-week, week number
- Timers slice (stopwatch / countdown / pomodoro) as live GUI widgets (mcstumble/pinetools productivity)

## v0.3 — files
- File mode for every text tool (open a file, save the result) — GUI already
  planned with native dialogs; hashes for *files* (streaming, not whole-file RAM)
- Image basics that overlap ConvertDesk (decide with SEAN: integrate or link)
- PDF text extract / merge / split (mcstumble PDF set)

## Later / needs a decision from SEAN
- Per-language code formatters (formatter.org): real parsers per language —
  big, per-language cost; recommend doing the top 5 languages only
- Text-to-speech / speech-to-text (needs Windows speech APIs)
- Games from alotoftools (2048, minesweeper…) — fun, but a different product;
  recommend NOT adding to a tools program
