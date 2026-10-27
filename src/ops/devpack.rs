//! The developer pack: eight power tools for developer workflows — hash
//! identification, JSONPath queries, a basic JSON Schema check, TOML ⇄ JSON,
//! JSON → TypeScript types, curl → code, and unified-diff export.
//! Every parser here is a documented basic subset, not a full implementation;
//! anything outside the subset gets a plain-words error, never a silent guess.

use super::Op;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{Map, Value};

fn op(
    id: &'static str,
    name: &'static str,
    cat: &'static str,
    desc: &'static str,
    hint: Option<&'static str>,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: cat,
        description: desc,
        param_hint: hint,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    vec![
        op("dev-hash-identify", "Hash Identifier", "Developer / Format", "Guess which algorithm most likely made a hash, from its length and shape. A best guess from length alone — never a proof.", None, hash_identify),
        op("dev-jsonpath", "JSONPath Query", "Developer / Format", "Pull values out of JSON with a path like $.store.book[0].title. Supports .key, ['key'], [n], [*] wildcards and ..key (find a key anywhere).", Some("JSONPath, e.g. $.store.book[0].title"), jsonpath_query),
        op("dev-json-schema", "JSON Schema Check (basic)", "Developer / Format", "Check JSON data (Input) against a JSON Schema (Parameter). Basic subset only: type, required, properties, items, enum, minimum/maximum, minLength/maxLength, minItems/maxItems. Unsupported keywords are ignored.", Some("JSON Schema"), json_schema_check),
        op("dev-toml-to-json", "TOML → JSON (basic)", "Developer / Format", "Convert TOML to JSON. Basic subset: comments, key = value (strings, numbers, booleans, arrays), [table] and [a.b] headers, [[array]] tables. No datetimes, inline tables or multiline strings.", None, toml_to_json),
        op("dev-json-to-toml", "JSON → TOML", "Developer / Format", "Convert a JSON object to TOML: plain values first, then nested objects as [tables] and arrays of objects as [[tables]].", None, json_to_toml),
        op("dev-json-to-ts", "JSON → TypeScript Types", "Developer / Format", "Work out TypeScript types from example JSON: the root object becomes an interface, and arrays of objects merge their keys (a key missing from some items becomes optional).", None, json_to_typescript),
        op("dev-curl", "curl → Code", "Developer / Format", "Turn a curl command into Python (requests) and JavaScript (fetch) code. Handles the URL, -X, -H, -d, -u and -b; other flags are skipped.", None, curl_to_code),
        op("an-diff-unified", "Unified Diff (Patch) Export", "Analyse", "Compare the Input text against the Parameter text and output a standard unified diff (the patch format other tools and Git understand), with 3 lines of context.", Some("second text"), unified_diff),
    ]
}

// ---------------------------------------------------------------------------
// 1. Hash Identifier
// ---------------------------------------------------------------------------

fn hash_identify(input: &str, _param: &str) -> Result<String, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Paste a hash into the Input box first.".into());
    }
    // Password hashes announce themselves with a $ prefix.
    let dollar: Option<(&str, &str)> =
        if s.starts_with("$2a$") || s.starts_with("$2b$") || s.starts_with("$2y$") {
            Some(("bcrypt", "the number after the prefix is its cost factor"))
        } else if s.starts_with("$argon2id$") {
            Some((
                "Argon2id",
                "a modern password hash — the parameters are encoded in the string itself",
            ))
        } else if s.starts_with("$argon2") {
            Some((
                "Argon2",
                "a modern password hash — the parameters are encoded in the string itself",
            ))
        } else if s.starts_with("$6$") {
            Some((
                "SHA-512 crypt (sha512crypt)",
                "the password-hash format used in /etc/shadow on Linux",
            ))
        } else if s.starts_with("$5$") {
            Some((
                "SHA-256 crypt (sha256crypt)",
                "the password-hash format used in /etc/shadow on Linux",
            ))
        } else if s.starts_with("$1$") {
            Some((
                "MD5 crypt (md5crypt)",
                "an old password-hash format found in older /etc/shadow files",
            ))
        } else {
            None
        };
    if let Some((name, note)) = dollar {
        return Ok(format!(
            "This is most likely {name} — {note}.\n\nLength: {} characters. Hashes like this carry their own salt and settings inside the string, so the prefix is a reliable tell.",
            s.chars().count()
        ));
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("This doesn't look like a hash: it has characters that aren't hex digits (0-9, a-f) and doesn't start with a known password-hash prefix like $2b$ (bcrypt) or $argon2.".into());
    }
    let len = s.len();
    let case =
        if s.chars().any(|c| c.is_ascii_uppercase()) && s.chars().any(|c| c.is_ascii_lowercase()) {
            "mixed-case hex"
        } else if s.chars().any(|c| c.is_ascii_uppercase()) {
            "uppercase hex"
        } else {
            "lowercase hex"
        };
    let candidates: &[(&str, &str)] = match len {
        8 => &[
            (
                "CRC32",
                "a 32-bit checksum for spotting errors — not a security hash",
            ),
            ("Adler-32", "a 32-bit checksum used by zlib"),
        ],
        16 => &[(
            "a 64-bit checksum (CRC-64 or similar)",
            "xxHash64 and other fast 64-bit hashes also print as 16 hex characters",
        )],
        32 => &[
            (
                "MD5",
                "very common — file checksums and old password stores",
            ),
            (
                "NTLM",
                "the Windows password hash — exactly the same length as MD5",
            ),
            (
                "MD4",
                "older; rarely seen outside Windows and legacy systems",
            ),
        ],
        40 => &[
            ("SHA-1", "used by Git commits and older certificates"),
            ("RIPEMD-160", "seen in some Bitcoin addresses"),
        ],
        56 => &[
            ("SHA-224", "a shortened member of the SHA-2 family"),
            ("SHA3-224", "the SHA-3 family at 224 bits"),
        ],
        64 => &[
            (
                "SHA-256",
                "the most common modern hash — Bitcoin, TLS, file checksums",
            ),
            ("SHA3-256", "the SHA-3 family at the same output size"),
            ("BLAKE2s", "a fast modern hash, 256-bit form"),
        ],
        96 => &[
            ("SHA-384", "often used in TLS certificates"),
            ("SHA3-384", "the SHA-3 family at 384 bits"),
        ],
        128 => &[
            ("SHA-512", "common for password storage and checksums"),
            ("SHA3-512", "the SHA-3 family at its largest standard size"),
            ("BLAKE2b", "a fast modern hash, 512-bit form"),
        ],
        _ => &[],
    };
    if candidates.is_empty() {
        return Ok(format!(
            "It is hex, {len} characters ({case}) — but no standard hash has this output length. The standard lengths are 8, 16, 32, 40, 56, 64, 96 and 128 hex characters, so this may be trimmed, padded, or not a plain hash at all."
        ));
    }
    let mut out = format!("Most likely, from length alone ({len} hex characters, {case}):\n");
    for (name, note) in candidates {
        out.push_str(&format!("- {name}: {note}\n"));
    }
    out.push_str("\nLength alone can't prove which one it is — several algorithms share each length. If you know where the hash came from, that settles it.");
    Ok(out)
}

// ---------------------------------------------------------------------------
// 2. JSONPath Query (subset)
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Seg {
    Key(String),
    Index(usize),
    Wildcard,
    DeepKey(String),
}

fn jsonpath_query(input: &str, param: &str) -> Result<String, String> {
    let root: Value =
        serde_json::from_str(input).map_err(|e| format!("Input is not valid JSON: {e}"))?;
    let segs = parse_path(param)?;
    let matches = eval_path(&root, &segs);
    if matches.is_empty() {
        return Ok("No matches.".into());
    }
    serde_json::to_string_pretty(&Value::Array(matches)).map_err(|e| e.to_string())
}

fn parse_path(path: &str) -> Result<Vec<Seg>, String> {
    let p = path.trim();
    if p.is_empty() {
        return Err("Enter a JSONPath as the Parameter, e.g. $.store.book[0].title".into());
    }
    let chars: Vec<char> = p.chars().collect();
    if chars.first() != Some(&'$') {
        return Err(format!(
            "A JSONPath starts with $ for the root — '{p}' doesn't"
        ));
    }
    let mut segs = Vec::new();
    let mut i = 1;
    while i < chars.len() {
        match chars[i] {
            '.' => {
                if chars.get(i + 1) == Some(&'.') {
                    let (name, next) = read_name(&chars, i + 2);
                    if name.is_empty() {
                        return Err(format!("In '{p}': '..' must be followed by a key name"));
                    }
                    if name == "*" {
                        segs.push(Seg::Wildcard);
                    } else {
                        segs.push(Seg::DeepKey(name));
                    }
                    i = next;
                } else {
                    let (name, next) = read_name(&chars, i + 1);
                    if name.is_empty() {
                        return Err(format!(
                            "In '{p}': '.' at position {i} must be followed by a key name"
                        ));
                    }
                    if name == "*" {
                        segs.push(Seg::Wildcard);
                    } else {
                        segs.push(Seg::Key(name));
                    }
                    i = next;
                }
            }
            '[' => {
                let (seg, next) = parse_bracket(&chars, i, p)?;
                segs.push(seg);
                i = next;
            }
            c => {
                return Err(format!(
                    "In '{p}': unexpected '{c}' at position {i} — expected '.' or '['"
                ));
            }
        }
    }
    Ok(segs)
}

fn read_name(chars: &[char], start: usize) -> (String, usize) {
    let mut j = start;
    let mut s = String::new();
    while j < chars.len() && chars[j] != '.' && chars[j] != '[' {
        s.push(chars[j]);
        j += 1;
    }
    (s, j)
}

fn parse_bracket(chars: &[char], start: usize, whole: &str) -> Result<(Seg, usize), String> {
    let bad = |why: &str| format!("In '{whole}': bad bracket segment at position {start} — {why}");
    let mut j = start + 1;
    match chars.get(j) {
        None => Err(bad("the '[' is never closed")),
        Some('*') => {
            j += 1;
            if chars.get(j) == Some(&']') {
                Ok((Seg::Wildcard, j + 1))
            } else {
                Err(bad("expected ']' after '*'"))
            }
        }
        Some(&q) if q == '\'' || q == '"' => {
            j += 1;
            let mut key = String::new();
            while j < chars.len() && chars[j] != q {
                key.push(chars[j]);
                j += 1;
            }
            if j >= chars.len() {
                return Err(bad("the quoted key is never closed"));
            }
            j += 1; // closing quote
            if chars.get(j) == Some(&']') {
                Ok((Seg::Key(key), j + 1))
            } else {
                Err(bad("expected ']' after the quoted key"))
            }
        }
        Some(c) if c.is_ascii_digit() => {
            let mut digits = String::new();
            while j < chars.len() && chars[j].is_ascii_digit() {
                digits.push(chars[j]);
                j += 1;
            }
            if chars.get(j) != Some(&']') {
                return Err(bad("expected ']' after the index number"));
            }
            let n: usize = digits
                .parse()
                .map_err(|_| bad("that index number is too large"))?;
            Ok((Seg::Index(n), j + 1))
        }
        Some(c) => Err(bad(&format!(
            "'[{c}' isn't supported — use [n], ['key'] or [*]"
        ))),
    }
}

fn eval_path(root: &Value, segs: &[Seg]) -> Vec<Value> {
    let mut current = vec![root.clone()];
    for seg in segs {
        let mut next = Vec::new();
        for v in &current {
            match seg {
                Seg::Key(k) => {
                    if let Some(x) = v.get(k.as_str()) {
                        next.push(x.clone());
                    }
                }
                Seg::Index(n) => {
                    if let Some(x) = v.get(*n) {
                        next.push(x.clone());
                    }
                }
                Seg::Wildcard => match v {
                    Value::Array(a) => next.extend(a.iter().cloned()),
                    Value::Object(m) => next.extend(m.values().cloned()),
                    _ => {}
                },
                Seg::DeepKey(k) => collect_deep(v, k, &mut next),
            }
        }
        current = next;
    }
    current
}

fn collect_deep(v: &Value, key: &str, out: &mut Vec<Value>) {
    match v {
        Value::Object(m) => {
            if let Some(x) = m.get(key) {
                out.push(x.clone());
            }
            for x in m.values() {
                collect_deep(x, key, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                collect_deep(x, key, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// 3. JSON Schema Check (basic subset)
// ---------------------------------------------------------------------------

fn json_schema_check(input: &str, param: &str) -> Result<String, String> {
    if param.trim().is_empty() {
        return Err("Enter the JSON Schema as the Parameter.".into());
    }
    let data: Value =
        serde_json::from_str(input).map_err(|e| format!("Input is not valid JSON: {e}"))?;
    let schema: Value =
        serde_json::from_str(param).map_err(|e| format!("The schema is not valid JSON: {e}"))?;
    let mut problems = Vec::new();
    validate(&data, &schema, "$", &mut problems);
    if problems.is_empty() {
        Ok("Valid ✓".into())
    } else {
        Ok(format!(
            "Not valid — {} problem(s):\n{}",
            problems.len(),
            problems.join("\n")
        ))
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn is_integer(v: &Value) -> bool {
    match v {
        Value::Number(n) => {
            n.is_i64() || n.is_u64() || n.as_f64().is_some_and(|f| f.fract() == 0.0)
        }
        _ => false,
    }
}

fn validate(data: &Value, schema: &Value, path: &str, problems: &mut Vec<String>) {
    let Some(s) = schema.as_object() else {
        return; // non-object schemas are outside the subset — ignored
    };
    if let Some(t) = s.get("type").and_then(|v| v.as_str()) {
        let ok = match t {
            "string" => data.is_string(),
            "number" => data.is_number(),
            "integer" => is_integer(data),
            "boolean" => data.is_boolean(),
            "object" => data.is_object(),
            "array" => data.is_array(),
            "null" => data.is_null(),
            _ => true, // unknown type name — ignored
        };
        if !ok {
            problems.push(format!("{path}: expected {t}, got {}", type_name(data)));
            return;
        }
    }
    if let Some(allowed) = s.get("enum").and_then(|v| v.as_array()) {
        if !allowed.contains(data) {
            problems.push(format!(
                "{path}: value is not one of the allowed enum values"
            ));
        }
    }
    if let Some(n) = data.as_f64() {
        if let Some(min) = s.get("minimum").and_then(|v| v.as_f64()) {
            if n < min {
                problems.push(format!("{path}: {n} is below the minimum {min}"));
            }
        }
        if let Some(max) = s.get("maximum").and_then(|v| v.as_f64()) {
            if n > max {
                problems.push(format!("{path}: {n} is above the maximum {max}"));
            }
        }
    }
    if let Some(text) = data.as_str() {
        let len = text.chars().count();
        if let Some(min) = s.get("minLength").and_then(|v| v.as_u64()) {
            if (len as u64) < min {
                problems.push(format!(
                    "{path}: string has {len} characters, fewer than minLength {min}"
                ));
            }
        }
        if let Some(max) = s.get("maxLength").and_then(|v| v.as_u64()) {
            if (len as u64) > max {
                problems.push(format!(
                    "{path}: string has {len} characters, more than maxLength {max}"
                ));
            }
        }
    }
    if let Some(arr) = data.as_array() {
        if let Some(min) = s.get("minItems").and_then(|v| v.as_u64()) {
            if (arr.len() as u64) < min {
                problems.push(format!(
                    "{path}: array has {} items, fewer than minItems {min}",
                    arr.len()
                ));
            }
        }
        if let Some(max) = s.get("maxItems").and_then(|v| v.as_u64()) {
            if (arr.len() as u64) > max {
                problems.push(format!(
                    "{path}: array has {} items, more than maxItems {max}",
                    arr.len()
                ));
            }
        }
        if let Some(items) = s.get("items") {
            if items.is_object() {
                for (idx, el) in arr.iter().enumerate() {
                    validate(el, items, &format!("{path}[{idx}]"), problems);
                }
            }
        }
    }
    if let Some(obj) = data.as_object() {
        if let Some(required) = s.get("required").and_then(|v| v.as_array()) {
            for r in required {
                if let Some(key) = r.as_str() {
                    if !obj.contains_key(key) {
                        problems.push(format!("{path}: missing required property '{key}'"));
                    }
                }
            }
        }
        if let Some(props) = s.get("properties").and_then(|v| v.as_object()) {
            for (key, sub) in props {
                if let Some(val) = obj.get(key) {
                    validate(val, sub, &format!("{path}.{key}"), problems);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. TOML → JSON (basic subset)
// ---------------------------------------------------------------------------

/// Split on `sep` wherever it appears outside quotes and brackets.
fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    let mut depth = 0usize;
    for c in s.chars() {
        if in_basic {
            cur.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_basic = false;
            }
        } else if in_literal {
            cur.push(c);
            if c == '\'' {
                in_literal = false;
            }
        } else {
            match c {
                '"' => {
                    in_basic = true;
                    cur.push(c);
                }
                '\'' => {
                    in_literal = true;
                    cur.push(c);
                }
                '[' | '{' => {
                    depth += 1;
                    cur.push(c);
                }
                ']' | '}' => {
                    depth = depth.saturating_sub(1);
                    cur.push(c);
                }
                c if c == sep && depth == 0 => parts.push(std::mem::take(&mut cur)),
                _ => cur.push(c),
            }
        }
    }
    parts.push(cur);
    parts
}

fn strip_toml_comment(line: &str) -> &str {
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_basic = false;
            }
        } else if in_literal {
            if c == '\'' {
                in_literal = false;
            }
        } else {
            match c {
                '"' => in_basic = true,
                '\'' => in_literal = true,
                '#' => return &line[..i],
                _ => {}
            }
        }
    }
    line
}

fn find_toml_eq(line: &str) -> Option<usize> {
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_basic = false;
            }
        } else if in_literal {
            if c == '\'' {
                in_literal = false;
            }
        } else {
            match c {
                '"' => in_basic = true,
                '\'' => in_literal = true,
                '=' => return Some(i),
                _ => {}
            }
        }
    }
    None
}

fn parse_key_path(s: &str, line: usize) -> Result<Vec<String>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err(format!("Missing key name (line {line})"));
    }
    let mut parts = Vec::new();
    for piece in split_top_level(s, '.') {
        let p = piece.trim();
        if p.is_empty() {
            return Err(format!("Empty key in '{s}' (line {line})"));
        }
        let quoted = p.len() >= 2
            && ((p.starts_with('"') && p.ends_with('"'))
                || (p.starts_with('\'') && p.ends_with('\'')));
        if quoted {
            parts.push(p[1..p.len() - 1].to_string());
        } else {
            parts.push(p.to_string());
        }
    }
    Ok(parts)
}

/// Walk/create the table at `path`. Arrays (from [[table]] headers) resolve
/// to their most recent element, matching TOML's meaning.
fn resolve_table_mut<'a>(
    table: &'a mut Map<String, Value>,
    path: &[String],
    line: usize,
) -> Result<&'a mut Map<String, Value>, String> {
    if path.is_empty() {
        return Ok(table);
    }
    let (first, rest) = path.split_first().unwrap();
    if !table.contains_key(first) {
        table.insert(first.clone(), Value::Object(Map::new()));
    }
    match table.get_mut(first) {
        Some(Value::Object(m)) => resolve_table_mut(m, rest, line),
        Some(Value::Array(a)) => match a.last_mut() {
            Some(Value::Object(m)) => resolve_table_mut(m, rest, line),
            _ => Err(format!("'{first}' is not a table (line {line})")),
        },
        _ => Err(format!(
            "Key '{first}' clashes with a value set earlier (line {line})"
        )),
    }
}

fn append_table(root: &mut Map<String, Value>, path: &[String], line: usize) -> Result<(), String> {
    let Some((last, parents)) = path.split_last() else {
        return Err(format!("Missing array-table name (line {line})"));
    };
    let parent = resolve_table_mut(root, parents, line)?;
    if !parent.contains_key(last) {
        parent.insert(last.clone(), Value::Array(Vec::new()));
    }
    match parent.get_mut(last) {
        Some(Value::Array(a)) => {
            a.push(Value::Object(Map::new()));
            Ok(())
        }
        _ => Err(format!(
            "'{last}' is already a value or a table, not an array of tables (line {line})"
        )),
    }
}

/// Parse a TOML "..." string starting at index 0 of `s`.
/// Returns the string and the rest of `s` after the closing quote.
fn parse_basic_string(s: &str, line: usize) -> Result<(String, &str), String> {
    let mut out = String::new();
    let mut chars = s.char_indices();
    chars.next(); // opening quote
                  // Byte position just past the current char; char_indices already
                  // accounts for escape chars consumed via chars.next() below.
    let mut i;
    while let Some((idx, c)) = chars.next() {
        i = idx + c.len_utf8();
        match c {
            '"' => return Ok((out, &s[i..])),
            '\\' => {
                let (_, e) = chars
                    .next()
                    .ok_or_else(|| format!("Unfinished escape in string (line {line})"))?;
                match e {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'u' | 'U' => {
                        let n = if e == 'u' { 4 } else { 8 };
                        let mut code = String::new();
                        for _ in 0..n {
                            match chars.next() {
                                Some((_, h)) => {
                                    code.push(h);
                                }
                                None => {
                                    return Err(format!(
                                        "Unfinished \\{e} escape in string (line {line})"
                                    ))
                                }
                            }
                        }
                        let cp = u32::from_str_radix(&code, 16)
                            .map_err(|_| format!("Bad \\{e} escape '\\{e}{code}' (line {line})"))?;
                        out.push(
                            char::from_u32(cp)
                                .ok_or_else(|| format!("Bad character code {cp} (line {line})"))?,
                        );
                    }
                    _ => return Err(format!("Unknown escape '\\{e}' in string (line {line})")),
                }
            }
            _ => out.push(c),
        }
    }
    Err(format!("String is never closed (line {line})"))
}

fn parse_toml_scalar(s: &str, line: usize) -> Result<Value, String> {
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }
    let b = s.as_bytes();
    let looks_date = s.len() >= 10
        && b.get(4) == Some(&b'-')
        && b.get(7) == Some(&b'-')
        && s[..4].chars().all(|c| c.is_ascii_digit());
    let looks_time = s.len() >= 5 && b.get(2) == Some(&b':');
    if looks_date || looks_time {
        return Err(format!(
            "Dates and times are not supported by this basic converter (line {line})"
        ));
    }
    let cleaned: String = s.chars().filter(|c| *c != '_').collect();
    if let Ok(n) = cleaned.parse::<i64>() {
        return Ok(Value::from(n));
    }
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        if let Some(rest) = cleaned.strip_prefix(prefix) {
            if let Ok(n) = i64::from_str_radix(rest, radix) {
                return Ok(Value::from(n));
            }
        }
    }
    if let Ok(f) = cleaned.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(Value::Number(n));
        }
    }
    Err(format!(
        "Could not understand the value '{s}' (line {line})"
    ))
}

fn parse_toml_value(s: &str, line: usize) -> Result<Value, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err(format!("Missing value (line {line})"));
    }
    if s.starts_with("\"\"\"") || s.starts_with("'''") {
        return Err(format!(
            "Multiline strings are not supported by this basic converter (line {line})"
        ));
    }
    if s.starts_with('{') {
        return Err(format!(
            "Inline tables are not supported by this basic converter (line {line})"
        ));
    }
    match s.chars().next().unwrap() {
        '"' => {
            let (value, rest) = parse_basic_string(s, line)?;
            if !rest.trim().is_empty() {
                return Err(format!(
                    "Unexpected text after the closing quote (line {line})"
                ));
            }
            Ok(Value::String(value))
        }
        '\'' => {
            let rel = s[1..]
                .find('\'')
                .ok_or_else(|| format!("String is never closed (line {line})"))?;
            let rest = &s[1 + rel + 1..];
            if !rest.trim().is_empty() {
                return Err(format!(
                    "Unexpected text after the closing quote (line {line})"
                ));
            }
            Ok(Value::String(s[1..1 + rel].to_string()))
        }
        '[' => {
            if !s.ends_with(']') {
                return Err(format!(
                    "Arrays must fit on one line in this basic converter (line {line})"
                ));
            }
            let inner = &s[1..s.len() - 1];
            let mut items = Vec::new();
            for piece in split_top_level(inner, ',') {
                let p = piece.trim();
                if !p.is_empty() {
                    items.push(parse_toml_value(p, line)?);
                }
            }
            Ok(Value::Array(items))
        }
        _ => parse_toml_scalar(s, line),
    }
}

fn toml_to_json(input: &str, _param: &str) -> Result<String, String> {
    let mut root = Map::new();
    let mut current_path: Vec<String> = Vec::new();
    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_toml_comment(raw).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if let Some(inner) = line.strip_prefix("[[").and_then(|l| l.strip_suffix("]]")) {
            let path = parse_key_path(inner, line_no)?;
            append_table(&mut root, &path, line_no)?;
            current_path = path;
        } else if let Some(inner) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let path = parse_key_path(inner, line_no)?;
            resolve_table_mut(&mut root, &path, line_no)?;
            current_path = path;
        } else if line.starts_with('[') {
            return Err(format!("Bad table header on line {line_no}: '{line}'"));
        } else {
            let eq = find_toml_eq(&line)
                .ok_or_else(|| format!("Line {line_no} is not a key = value pair: '{line}'"))?;
            let key_path = parse_key_path(line[..eq].trim(), line_no)?;
            let value = parse_toml_value(line[eq + 1..].trim(), line_no)?;
            let table = resolve_table_mut(&mut root, &current_path, line_no)?;
            let Some((last, parents)) = key_path.split_last() else {
                return Err(format!("Missing key name (line {line_no})"));
            };
            let target = resolve_table_mut(table, parents, line_no)?;
            if target.contains_key(last) {
                return Err(format!("Duplicate key '{last}' (line {line_no})"));
            }
            target.insert(last.clone(), value);
        }
    }
    serde_json::to_string_pretty(&Value::Object(root)).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 5. JSON → TOML
// ---------------------------------------------------------------------------

fn toml_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn toml_key(k: &str) -> String {
    if !k.is_empty()
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        k.to_string()
    } else {
        format!("\"{}\"", toml_escape(k))
    }
}

fn dotted(path: &[String]) -> String {
    path.iter()
        .map(|k| toml_key(k))
        .collect::<Vec<_>>()
        .join(".")
}

/// Format a scalar, or an array of scalars/arrays, as a TOML value.
fn toml_fmt(v: &Value) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(format!("\"{}\"", toml_escape(s))),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Err("TOML has no null value — remove or replace the nulls first.".into()),
        Value::Array(a) => {
            let mut parts = Vec::new();
            for el in a {
                parts.push(toml_fmt(el)?);
            }
            Ok(format!("[{}]", parts.join(", ")))
        }
        Value::Object(_) => {
            Err("An object inside an array can't be written as a plain TOML value.".into())
        }
    }
}

#[derive(PartialEq)]
enum TomlKind {
    Value,
    Table,
    TableArray,
}

fn toml_kind(v: &Value) -> Result<TomlKind, String> {
    match v {
        Value::Null => Err("TOML has no null value — remove or replace the nulls first.".into()),
        Value::Object(_) => Ok(TomlKind::Table),
        Value::Array(a) => {
            if !a.is_empty() && a.iter().all(|x| x.is_object()) {
                Ok(TomlKind::TableArray)
            } else if a.iter().any(|x| x.is_object()) {
                Err("TOML can't mix objects and plain values in one array.".into())
            } else {
                Ok(TomlKind::Value)
            }
        }
        _ => Ok(TomlKind::Value),
    }
}

fn emit_table(
    map: &Map<String, Value>,
    path: &[String],
    lines: &mut Vec<String>,
) -> Result<(), String> {
    for (k, v) in map {
        if toml_kind(v)? == TomlKind::Value {
            lines.push(format!("{} = {}", toml_key(k), toml_fmt(v)?));
        }
    }
    for (k, v) in map {
        if toml_kind(v)? == TomlKind::Table {
            let mut p = path.to_vec();
            p.push(k.clone());
            lines.push(String::new());
            lines.push(format!("[{}]", dotted(&p)));
            emit_table(v.as_object().unwrap(), &p, lines)?;
        }
    }
    for (k, v) in map {
        if toml_kind(v)? == TomlKind::TableArray {
            let mut p = path.to_vec();
            p.push(k.clone());
            if let Value::Array(items) = v {
                for el in items {
                    lines.push(String::new());
                    lines.push(format!("[[{}]]", dotted(&p)));
                    emit_table(el.as_object().unwrap(), &p, lines)?;
                }
            }
        }
    }
    Ok(())
}

fn json_to_toml(input: &str, _param: &str) -> Result<String, String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| format!("Input is not valid JSON: {e}"))?;
    let Some(obj) = v.as_object() else {
        return Err(
            "Root must be a JSON object — a TOML file is a set of key = value pairs.".into(),
        );
    };
    let mut lines = Vec::new();
    emit_table(obj, &[], &mut lines)?;
    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// 6. JSON → TypeScript Types
// ---------------------------------------------------------------------------

fn is_ts_ident(k: &str) -> bool {
    let mut chars = k.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn ts_object(entries: Vec<(String, String, bool)>, level: usize) -> String {
    if entries.is_empty() {
        return "{}".into();
    }
    let pad = "  ".repeat(level + 1);
    let close_pad = "  ".repeat(level);
    let mut parts = vec!["{".to_string()];
    for (key, ty, required) in entries {
        let shown = if is_ts_ident(&key) {
            key
        } else {
            format!("\"{key}\"")
        };
        let opt = if required { "" } else { "?" };
        parts.push(format!("{pad}{shown}{opt}: {ty};"));
    }
    parts.push(format!("{close_pad}}}"));
    parts.join("\n")
}

fn ts_merged_objects(items: &[Value], level: usize) -> String {
    let mut keys: Vec<String> = Vec::new();
    for el in items {
        if let Some(m) = el.as_object() {
            for k in m.keys() {
                if !keys.contains(k) {
                    keys.push(k.clone());
                }
            }
        }
    }
    let mut entries = Vec::new();
    for k in keys {
        let mut types: Vec<String> = Vec::new();
        let mut present = 0usize;
        for el in items {
            if let Some(val) = el.get(&k) {
                present += 1;
                let t = ts_type(val, level + 1);
                if !types.contains(&t) {
                    types.push(t);
                }
            }
        }
        entries.push((k, types.join(" | "), present == items.len()));
    }
    ts_object(entries, level)
}

fn ts_array(items: &[Value], level: usize) -> String {
    if items.is_empty() {
        return "unknown[]".into();
    }
    if items.iter().all(|x| x.is_object()) {
        return format!("{}[]", ts_merged_objects(items, level));
    }
    let mut types: Vec<String> = Vec::new();
    for el in items {
        let t = ts_type(el, level);
        if !types.contains(&t) {
            types.push(t);
        }
    }
    if types.len() == 1 {
        format!("{}[]", types[0])
    } else {
        format!("({})[]", types.join(" | "))
    }
}

fn ts_type(v: &Value, level: usize) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(_) => "boolean".into(),
        Value::Number(_) => "number".into(),
        Value::String(_) => "string".into(),
        Value::Array(a) => ts_array(a, level),
        Value::Object(m) => {
            let entries = m
                .iter()
                .map(|(k, val)| (k.clone(), ts_type(val, level + 1), true))
                .collect();
            ts_object(entries, level)
        }
    }
}

fn json_to_typescript(input: &str, _param: &str) -> Result<String, String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| format!("Input is not valid JSON: {e}"))?;
    match &v {
        Value::Object(m) => {
            let entries = m
                .iter()
                .map(|(k, val)| (k.clone(), ts_type(val, 1), true))
                .collect();
            Ok(format!("export interface Root {}", ts_object(entries, 0)))
        }
        _ => Ok(format!("export type Root = {};", ts_type(&v, 0))),
    }
}

// ---------------------------------------------------------------------------
// 7. curl → Code
// ---------------------------------------------------------------------------

struct CurlReq {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    data: Option<String>,
    user: Option<String>,
}

/// Split a command line into words, honouring 'single' and "double" quotes,
/// backslash escapes, and backslash-newline line continuations.
fn tokenize_curl(s: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut has_token = false;
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if in_single {
            if c == '\'' {
                in_single = false;
            } else {
                cur.push(c);
            }
        } else if in_double {
            match c {
                '"' => in_double = false,
                '\\' => {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                }
                _ => cur.push(c),
            }
        } else {
            match c {
                '\'' => {
                    in_single = true;
                    has_token = true;
                }
                '"' => {
                    in_double = true;
                    has_token = true;
                }
                '\\' => match chars.peek() {
                    Some('\n') => {
                        chars.next();
                    }
                    Some(&n) => {
                        cur.push(n);
                        chars.next();
                        has_token = true;
                    }
                    None => {}
                },
                c if c.is_whitespace() => {
                    if has_token || !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                        has_token = false;
                    }
                }
                _ => {
                    cur.push(c);
                    has_token = true;
                }
            }
        }
    }
    if in_single || in_double {
        return Err("A quote in the curl command is never closed.".into());
    }
    if has_token || !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}

fn parse_curl(tokens: &[String]) -> Result<CurlReq, String> {
    let mut i = 0;
    if tokens.first().map(|t| t.as_str()) == Some("$") {
        i = 1;
    }
    if tokens.get(i).map(|t| t.as_str()) != Some("curl") {
        return Err("Input should be a curl command — it needs to start with 'curl'.".into());
    }
    i += 1;
    let get_val = |i: &mut usize, inline: &Option<String>, flag: &str| -> Result<String, String> {
        if let Some(v) = inline {
            return Ok(v.clone());
        }
        *i += 1;
        tokens
            .get(*i)
            .cloned()
            .ok_or_else(|| format!("The flag {flag} needs a value after it"))
    };
    let mut method: Option<String> = None;
    let mut url: Option<String> = None;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut data: Option<String> = None;
    let mut user: Option<String> = None;
    while i < tokens.len() {
        let raw = tokens[i].clone();
        let (tok, inline_val): (String, Option<String>) = if raw.starts_with("--") {
            match raw.split_once('=') {
                Some((f, v)) => (f.to_string(), Some(v.to_string())),
                None => (raw.clone(), None),
            }
        } else {
            (raw.clone(), None)
        };
        match tok.as_str() {
            "-X" | "--request" => {
                method = Some(get_val(&mut i, &inline_val, &tok)?.to_uppercase());
            }
            "-H" | "--header" => {
                let h = get_val(&mut i, &inline_val, &tok)?;
                let (name, value) = h
                    .split_once(':')
                    .ok_or_else(|| format!("Header '{h}' has no ':' in it"))?;
                headers.push((name.trim().to_string(), value.trim().to_string()));
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii"
            | "--data-urlencode" => {
                data = Some(get_val(&mut i, &inline_val, &tok)?);
            }
            "-u" | "--user" => {
                user = Some(get_val(&mut i, &inline_val, &tok)?);
            }
            "-b" | "--cookie" => {
                let c = get_val(&mut i, &inline_val, &tok)?;
                headers.push(("Cookie".to_string(), c));
            }
            "-A" | "--user-agent" => {
                let a = get_val(&mut i, &inline_val, &tok)?;
                headers.push(("User-Agent".to_string(), a));
            }
            "--url" => {
                url = Some(get_val(&mut i, &inline_val, &tok)?);
            }
            "-o" | "--output" | "--connect-timeout" | "--max-time" | "-m" | "--retry" => {
                get_val(&mut i, &inline_val, &tok)?;
            }
            f if f.starts_with('-') => {
                // Boolean or unsupported flag — this basic converter skips it.
            }
            _ => {
                if url.is_none() {
                    url = Some(tok.clone());
                }
            }
        }
        i += 1;
    }
    let url = url.ok_or(
        "No URL found — a curl command needs a web address to request. Check the command and try again.",
    )?;
    let method = method.unwrap_or_else(|| {
        if data.is_some() {
            "POST".to_string()
        } else {
            "GET".to_string()
        }
    });
    Ok(CurlReq {
        method,
        url,
        headers,
        data,
        user,
    })
}

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn py_literal(v: &Value) -> String {
    match v {
        Value::String(s) => quoted(s),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        Value::Null => "None".into(),
        Value::Number(n) => n.to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(py_literal).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, val)| format!("{}: {}", quoted(k), py_literal(val)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_python(r: &CurlReq) -> String {
    let mut lines = vec!["import requests".to_string(), String::new()];
    let mut args: Vec<String> = Vec::new();
    if !r.headers.is_empty() {
        lines.push("headers = {".into());
        for (k, v) in &r.headers {
            lines.push(format!("    {}: {},", quoted(k), quoted(v)));
        }
        lines.push("}".into());
        lines.push(String::new());
        args.push("headers=headers".into());
    }
    if let Some(d) = &r.data {
        match serde_json::from_str::<Value>(d) {
            Ok(v) => args.push(format!("json={}", py_literal(&v))),
            Err(_) => args.push(format!("data={}", quoted(d))),
        }
    }
    if let Some(u) = &r.user {
        let (name, pass) = u.split_once(':').unwrap_or((u.as_str(), ""));
        args.push(format!("auth=({}, {})", quoted(name), quoted(pass)));
    }
    let call = format!("requests.{}", r.method.to_lowercase());
    if args.is_empty() {
        lines.push(format!("response = {call}({})", quoted(&r.url)));
    } else {
        lines.push(format!("response = {call}("));
        lines.push(format!("    {},", quoted(&r.url)));
        for a in &args {
            lines.push(format!("    {a},"));
        }
        lines.push(")".into());
    }
    lines.push("print(response.text)".into());
    lines.join("\n")
}

fn render_js(r: &CurlReq) -> String {
    let mut headers: Vec<(String, String)> = r.headers.clone();
    if let Some(u) = &r.user {
        headers.push((
            "Authorization".to_string(),
            format!("Basic {}", STANDARD.encode(u)),
        ));
    }
    let mut opts: Vec<String> = vec![format!("  method: {},", quoted(&r.method))];
    if !headers.is_empty() {
        opts.push("  headers: {".into());
        for (k, v) in &headers {
            opts.push(format!("    {}: {},", quoted(k), quoted(v)));
        }
        opts.push("  },".into());
    }
    if let Some(d) = &r.data {
        match serde_json::from_str::<Value>(d) {
            Ok(v) => opts.push(format!(
                "  body: JSON.stringify({}),",
                serde_json::to_string(&v).unwrap_or_default()
            )),
            Err(_) => opts.push(format!("  body: {},", quoted(d))),
        }
    }
    let mut lines = vec![format!(
        "const response = await fetch({}, {{",
        quoted(&r.url)
    )];
    lines.extend(opts);
    lines.push("});".into());
    lines.push("console.log(await response.text());".into());
    lines.join("\n")
}

fn curl_to_code(input: &str, _param: &str) -> Result<String, String> {
    let tokens = tokenize_curl(input.trim())?;
    if tokens.is_empty() {
        return Err("Paste a curl command into the Input box first.".into());
    }
    let req = parse_curl(&tokens)?;
    Ok(format!(
        "Python (requests):\n{}\n\nJavaScript (fetch):\n{}",
        render_python(&req),
        render_js(&req)
    ))
}

// ---------------------------------------------------------------------------
// 8. Unified Diff (Patch) Export
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum DiffLine {
    Same,
    Del,
    Add,
}

fn unified_diff(a: &str, b: &str) -> Result<String, String> {
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    let (n, m) = (al.len(), bl.len());
    // Longest-common-subsequence table, same approach as the plain diff tool.
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for x in (0..n).rev() {
        for y in (0..m).rev() {
            dp[x][y] = if al[x] == bl[y] {
                dp[x + 1][y + 1] + 1
            } else {
                dp[x + 1][y].max(dp[x][y + 1])
            };
        }
    }
    let mut ops: Vec<(DiffLine, &str)> = Vec::new();
    let (mut x, mut y) = (0, 0);
    while x < n && y < m {
        if al[x] == bl[y] {
            ops.push((DiffLine::Same, al[x]));
            x += 1;
            y += 1;
        } else if dp[x + 1][y] >= dp[x][y + 1] {
            ops.push((DiffLine::Del, al[x]));
            x += 1;
        } else {
            ops.push((DiffLine::Add, bl[y]));
            y += 1;
        }
    }
    while x < n {
        ops.push((DiffLine::Del, al[x]));
        x += 1;
    }
    while y < m {
        ops.push((DiffLine::Add, bl[y]));
        y += 1;
    }
    if ops.iter().all(|(k, _)| *k == DiffLine::Same) {
        return Ok("No differences.".into());
    }
    const CTX: usize = 3;
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (idx, (kind, _)) in ops.iter().enumerate() {
        if *kind == DiffLine::Same {
            continue;
        }
        let start = idx.saturating_sub(CTX);
        let end = (idx + CTX).min(ops.len() - 1);
        let mut merged = false;
        if let Some(last) = ranges.last_mut() {
            if start <= last.1 + 1 {
                last.1 = last.1.max(end);
                merged = true;
            }
        }
        if !merged {
            ranges.push((start, end));
        }
    }
    let count_except = |slice: &[(DiffLine, &str)], skip: DiffLine| {
        slice.iter().filter(|(k, _)| *k != skip).count()
    };
    let mut out = vec!["--- a".to_string(), "+++ b".to_string()];
    for (start, end) in ranges {
        let old_before = count_except(&ops[..start], DiffLine::Add);
        let new_before = count_except(&ops[..start], DiffLine::Del);
        let hunk = &ops[start..=end];
        let old_count = count_except(hunk, DiffLine::Add);
        let new_count = count_except(hunk, DiffLine::Del);
        let old_start = if old_count == 0 {
            old_before
        } else {
            old_before + 1
        };
        let new_start = if new_count == 0 {
            new_before
        } else {
            new_before + 1
        };
        out.push(format!(
            "@@ -{old_start},{old_count} +{new_start},{new_count} @@"
        ));
        for (kind, line) in hunk {
            let prefix = match kind {
                DiffLine::Same => ' ',
                DiffLine::Del => '-',
                DiffLine::Add => '+',
            };
            out.push(format!("{prefix}{line}"));
        }
    }
    Ok(format!("{}\n", out.join("\n")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_identify_candidates_and_errors() {
        let md5 = hash_identify("d41d8cd98f00b204e9800998ecf8427e", "").unwrap();
        assert!(md5.contains("Most likely"));
        assert!(md5.contains("MD5"));
        assert!(md5.contains("NTLM"));
        let sha = hash_identify(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "",
        )
        .unwrap();
        assert!(sha.contains("SHA-256"));
        assert!(sha.contains("64 hex characters"));
        let bcrypt =
            hash_identify("$2b$12$abcdefghijklmnopqrstuuVZp8Zp8Zp8Zp8Zp8Zp8Zp8Zp8", "").unwrap();
        assert!(bcrypt.contains("bcrypt"));
        let crc = hash_identify("deadbeef", "").unwrap();
        assert!(crc.contains("CRC32"));
        assert!(hash_identify("not a hash!", "").is_err());
        assert!(hash_identify("   ", "").is_err());
    }

    #[test]
    fn jsonpath_subset_and_errors() {
        let json = r#"{"store":{"book":[{"title":"A"},{"title":"B"}],"name":"shop"}}"#;
        let first = jsonpath_query(json, "$.store.book[0].title").unwrap();
        assert_eq!(first, "[\n  \"A\"\n]");
        let all = jsonpath_query(json, "$.store.book[*].title").unwrap();
        assert!(all.contains("\"A\""));
        assert!(all.contains("\"B\""));
        let deep = jsonpath_query(json, "$..title").unwrap();
        assert!(deep.contains("\"A\""));
        assert!(deep.contains("\"B\""));
        assert_eq!(jsonpath_query(json, "$.store.nope").unwrap(), "No matches.");
        assert!(jsonpath_query(json, "store.book").is_err());
        assert!(jsonpath_query(json, "$.store.book[").is_err());
        assert!(jsonpath_query("not json", "$.a").is_err());
        assert!(jsonpath_query(json, "").is_err());
    }

    #[test]
    fn json_schema_check_reports_paths() {
        let schema = r#"{"type":"object","required":["name"],"properties":{"user":{"type":"object","properties":{"age":{"type":"integer","minimum":0}},"required":["age"]}}}"#;
        let good = json_schema_check(r#"{"name":"x","user":{"age":3}}"#, schema).unwrap();
        assert_eq!(good, "Valid ✓");
        let bad = json_schema_check(r#"{"user":{"age":"old"}}"#, schema).unwrap();
        assert!(bad.contains("$.user.age: expected integer, got string"));
        assert!(bad.contains("missing required property 'name'"));
        let range = json_schema_check(
            r#"{"n": 12}"#,
            r#"{"type":"object","properties":{"n":{"type":"number","maximum":10}}}"#,
        )
        .unwrap();
        assert!(range.contains("$.n: 12 is above the maximum 10"));
        assert!(json_schema_check("{}", "").is_err());
        assert!(json_schema_check("{}", "{oops").is_err());
    }

    #[test]
    fn toml_to_json_subset_and_errors() {
        let toml = "title = \"Demo\"\ncount = 3\npi = 3.5\nflags = [1, 2]\n\n[server]\nhost = \"localhost\"\nport = 8080\n\n[[items]]\nname = \"a\"\n\n[[items]]\nname = \"b\"\n";
        let out = toml_to_json(toml, "").unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["title"], Value::String("Demo".into()));
        assert_eq!(v["count"], Value::from(3));
        assert_eq!(v["server"]["port"], Value::from(8080));
        assert_eq!(v["items"][1]["name"], Value::String("b".into()));
        assert_eq!(v["flags"][1], Value::from(2));
        let err = toml_to_json("x = \"\"\"abc\"\"\"", "").unwrap_err();
        assert!(err.contains("Multiline strings"));
        assert!(err.contains("line 1"));
        let err2 = toml_to_json("when = 2024-01-31", "").unwrap_err();
        assert!(err2.contains("Dates and times"));
        assert!(toml_to_json("just words", "").is_err());
    }

    #[test]
    fn json_to_toml_layout_and_errors() {
        let out = json_to_toml(
            r#"{"name":"x","tags":["a","b"],"server":{"host":"h","port":80},"items":[{"a":1},{"a":2}]}"#,
            "",
        )
        .unwrap();
        assert!(out.contains("name = \"x\""));
        assert!(out.contains("tags = [\"a\", \"b\"]"));
        assert!(out.contains("[server]"));
        assert!(out.contains("port = 80"));
        assert_eq!(out.matches("[[items]]").count(), 2);
        // scalars must come before any table header
        assert!(out.find("name =").unwrap() < out.find("[server]").unwrap());
        let err = json_to_toml("[1, 2]", "").unwrap_err();
        assert!(err.contains("Root must be a JSON object"));
        assert!(json_to_toml(r#"{"a": null}"#, "").is_err());
        assert!(json_to_toml("{bad", "").is_err());
    }

    #[test]
    fn json_to_typescript_merges_array_objects() {
        let out = json_to_typescript(
            r#"{"name":"x","score":1.5,"tags":["a"],"items":[{"id":1},{"id":2,"extra":true}]}"#,
            "",
        )
        .unwrap();
        assert!(out.starts_with("export interface Root {"));
        assert!(out.contains("name: string;"));
        assert!(out.contains("score: number;"));
        assert!(out.contains("tags: string[];"));
        assert!(out.contains("id: number;"));
        assert!(out.contains("extra?: boolean;"));
        let arr = json_to_typescript("[1, 2]", "").unwrap();
        assert_eq!(arr, "export type Root = number[];");
        assert!(json_to_typescript("{bad", "").is_err());
    }

    #[test]
    fn curl_to_code_python_and_fetch() {
        let cmd = "curl -X POST https://api.example.com/v1/items -H \"Content-Type: application/json\" -H \"X-Key: abc\" -d '{\"a\": 1}' -u bob:secret";
        let out = curl_to_code(cmd, "").unwrap();
        assert!(out.contains("import requests"));
        assert!(out.contains("requests.post"));
        assert!(out.contains("\"Content-Type\": \"application/json\""));
        assert!(out.contains("json={\"a\": 1}"));
        assert!(out.contains("auth=(\"bob\", \"secret\")"));
        assert!(out.contains("await fetch(\"https://api.example.com/v1/items\""));
        assert!(out.contains("JSON.stringify({\"a\":1})"));
        assert!(out.contains("Basic Ym9iOnNlY3JldA=="));
        let get = curl_to_code("curl https://example.com", "").unwrap();
        assert!(get.contains("requests.get(\"https://example.com\")"));
        assert!(curl_to_code("curl -X GET", "").is_err());
        assert!(curl_to_code("wget https://example.com", "").is_err());
        assert!(curl_to_code("", "").is_err());
    }

    #[test]
    fn unified_diff_hunks_and_identical() {
        let a = "one\ntwo\nthree\nfour\nfive\nsix\nseven";
        let b = "one\ntwo\nthree\nFOUR\nfive\nsix\nseven";
        let out = unified_diff(a, b).unwrap();
        assert!(out.starts_with("--- a\n+++ b\n"));
        assert!(out.contains("@@ -1,7 +1,7 @@"));
        assert!(out.contains("-four"));
        assert!(out.contains("+FOUR"));
        assert!(out.contains(" three"));
        assert!(out.ends_with('\n'));
        assert_eq!(unified_diff(a, a).unwrap(), "No differences.");
        let added = unified_diff("x", "x\ny").unwrap();
        assert!(added.contains("+y"));
    }
}
