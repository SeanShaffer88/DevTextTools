//! The second developer pack: JWT signing, number/date forensics, a JSON
//! structural diff, INI and .env converters, mock data, SQL helpers, Git
//! helpers, a regex explainer, log/stack utilities, text-encoding forensics
//! and a line-ending inspector/converter. Every parser is a documented
//! basic subset — anything outside the subset gets a plain-words error,
//! never a silent guess.

use super::Op;
use crate::{split_param, Rng};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
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
        op("dev-jwt-build", "JWT Builder / Signer (HS256)", "Developer / Format", "Build and sign a JSON Web Token with HMAC-SHA256 (HS256). Input = the payload as a JSON object; Parameter = the secret. Add |||{...} after the secret for extra header fields. For testing only — a real login system needs proper key handling.", Some("secret|||extra header JSON (optional)"), jwt_build),
        op("dev-number-identify", "Mystery Number Identifier", "Developer / Format", "Paste any number and see every sensible reading at once: Unix time in seconds/milliseconds/microseconds, Windows FILETIME, an Excel serial date, an IPv4 address, a file size, hex, binary and ASCII.", None, number_identify),
        op("dev-json-diff", "JSON Structural Diff", "Developer / Format", "Compare two JSON documents by structure, not text: keys only in one side and values that changed, listed by path like $.items[2]. Input = first document, Parameter = second document.", Some("second JSON document"), json_diff),
        op("dev-ini-to-json", "INI / Properties → JSON", "Developer / Format", "Convert an INI or .properties file to JSON. [sections] and dotted keys nest into objects; every value stays a string (INI has no numbers or booleans).", None, ini_to_json),
        op("dev-json-to-ini", "JSON → INI", "Developer / Format", "Convert a JSON object to INI: top-level values first, nested objects become [sections] (deeper nesting uses dotted [a.b] section names), arrays are comma-joined.", None, json_to_ini),
        op("dev-env-to-json", "ENV → JSON", "Developer / Format", "Convert .env lines (KEY=VALUE, with an optional 'export ' prefix and quoted values) to a flat JSON object of strings.", None, env_to_json),
        op("dev-json-to-env", "JSON → ENV", "Developer / Format", "Convert a flat JSON object of plain values to .env lines. Values with spaces or special characters are double-quoted; nested objects and arrays can't be represented and error out.", None, json_to_env),
        op("dev-env-diff", ".env Compare", "Developer / Format", "Compare two .env files: keys only in the first, only in the second, and keys whose values differ. Input = first file, Parameter = second file (add |||mask to hide the differing values).", Some("second .env text|||mask (optional)"), env_diff),
        op("dev-env-merge", ".env Merge", "Developer / Format", "Merge two .env files into one sorted list. Input = base file, Parameter = overrides (the Parameter wins on conflicts). Comments are dropped.", Some("second .env text"), env_merge),
        op("dev-mock-data", "Mock Data Generator", "Developer / Format", "Fill a JSON template with made-up data: Parameter = how many rows (default 1, max 200). String values can use {{uuid}}, {{firstName}}, {{lastName}}, {{fullName}}, {{email}}, {{int:1:100}}, {{float:0:1}}, {{bool}}, {{word}}, {{city}}, {{company}}, {{url}}, {{ipv4}}, {{color}}, {{date}} and {{choice:a|b|c}}.", Some("row count (default 1)"), mock_data),
        op("dev-sql-in-list", "SQL IN List Builder", "Developer / Format", "Turn one value per line into a SQL IN list: ('a', 'b', 'c'), with quotes escaped. Set the Parameter to 'numbers' for unquoted numbers.", Some("numbers (optional)"), sql_in_list),
        op("dev-sql-create-table", "SQL CREATE TABLE from Data", "Developer / Format", "Work out a CREATE TABLE statement from data: paste CSV (first row = headers) or a JSON array of objects, and the column types (INTEGER, REAL, BOOLEAN, TEXT) are inferred from the values.", Some("table name (default my_table)"), sql_create_table),
        op("dev-sql-expand", "SQL Placeholder Expander", "Developer / Format", "Fill the ? placeholders in a SQL statement with real values from the Parameter (separated by |||), quoting text safely — handy for turning logged queries into runnable ones.", Some("value1|||value2|||…"), sql_expand),
        op("dev-gitignore", ".gitignore Generator", "Developer / Format", "Build a .gitignore from ready-made templates. Parameter = one or more of: rust, node, python, java, go, csharp, vscode, jetbrains, windows, macos, linux.", Some("rust, node, python, …"), gitignore_gen),
        op("dev-commit-check", "Commit Message Checker", "Developer / Format", "Check a commit message against the Conventional Commits rules — type(scope): subject, subject at most 72 characters, no trailing full stop, body separated by a blank line — with a verdict per rule.", None, commit_check),
        op("dev-changelog", "Changelog from Commits", "Developer / Format", "Turn a list of commits (one per line, with or without leading hashes) into a grouped Markdown changelog: Features, Fixes, Performance, Refactoring, Documentation, Other Changes.", None, changelog_from_commits),
        op("dev-regex-explain", "Regex Explainer", "Developer / Format", "Translate a regular expression into plain English, one line per element — literals, escapes, character classes, groups, quantifiers and alternation explained. A best-effort guide, not a validator.", None, regex_explain),
        op("an-log-analyze", "Log File Analyzer", "Analyse", "Summarise log text: line counts per level (errors, warnings, info, debug, trace), the busiest level, the most repeated lines (with numbers normalised away), and the first and last timestamps found.", None, log_analyze),
        op("an-stack-clean", "Stack Trace Cleaner", "Analyse", "Tidy a stack trace: runs of identical lines folded into one line with a ×N count, long file paths shortened to their last two segments, and a frame count reported at the end.", None, stack_clean),
        op("enc-mojibake-fix", "Mojibake Fixer (broken UTF-8)", "Encode / Decode", "Repair text that was UTF-8 but got read as Latin-1/Windows-1252 — the classic 'café' turning into 'cafÃ©'. Clean text comes back unchanged.", None, mojibake_fix),
        op("enc-invisible-reveal", "Invisible Character Revealer", "Encode / Decode", "Reveal invisible characters — zero-width spaces, directional marks, soft hyphens and friends — replacing each with a visible <U+200B …> tag, with a count at the top.", None, invisible_reveal),
        op("text-line-endings", "Line Endings (inspect / convert)", "Lines", "Inspect or convert line endings. With no Parameter it counts CRLF / LF / CR endings and says which dominates; set the Parameter to LF, CRLF or CR to convert everything to that ending.", Some("LF / CRLF / CR (empty = inspect)"), line_endings),
    ]
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// HMAC-SHA256, the standard ipad/opad construction (mirrors hashgen.rs).
fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut k = if key.len() > 64 {
        sha2::Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(64, 0);
    let ipad: Vec<u8> = k.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = k.iter().map(|b| b ^ 0x5c).collect();
    let mut inner = <sha2::Sha256 as sha2::Digest>::new();
    inner.update(&ipad);
    inner.update(msg);
    let inner_hash = inner.finalize();
    let mut outer = <sha2::Sha256 as sha2::Digest>::new();
    outer.update(&opad);
    outer.update(inner_hash);
    outer.finalize().to_vec()
}

// Unix time <-> civil date (Howard Hinnant's algorithms, mirrored from
// convert.rs so the format matches the rest of the program).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

/// Format Unix seconds as `YYYY-MM-DD HH:MM:SS UTC`.
fn fmt_unix(ts: i64) -> String {
    let days = ts.div_euclid(86400);
    let secs = ts.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Format a day count (days since 1970-01-01) as `YYYY-MM-DD`.
fn fmt_date(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

// ---------------------------------------------------------------------------
// 1. JWT Builder / Signer (HS256)
// ---------------------------------------------------------------------------

fn jwt_build(input: &str, param: &str) -> Result<String, String> {
    let parts = split_param(param);
    let secret = parts.first().map(|s| s.trim()).unwrap_or("");
    if secret.is_empty() {
        return Err(
            "Enter the secret as the Parameter (just the secret, or secret|||{extra header JSON})."
                .into(),
        );
    }
    let payload: Value = serde_json::from_str(input.trim())
        .map_err(|e| format!("The payload is not valid JSON: {e}"))?;
    if !payload.is_object() {
        return Err(
            "The payload must be a JSON object, like {\"sub\":\"123\",\"name\":\"Ada\"}.".into(),
        );
    }
    let mut header = Map::new();
    header.insert("alg".to_string(), Value::String("HS256".to_string()));
    header.insert("typ".to_string(), Value::String("JWT".to_string()));
    if let Some(extra) = parts.get(1).map(|s| s.trim()) {
        if !extra.is_empty() {
            let ev: Value = serde_json::from_str(extra)
                .map_err(|e| format!("The extra header is not valid JSON: {e}"))?;
            let eo = ev
                .as_object()
                .ok_or("The extra header must be a JSON object, like {\"kid\":\"key-1\"}.")?;
            for (k, v) in eo {
                header.insert(k.clone(), v.clone());
            }
            // We sign with HS256 whatever the extra header tried to claim.
            header.insert("alg".to_string(), Value::String("HS256".to_string()));
        }
    }
    let header_v = Value::Object(header);
    let signing = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(header_v.to_string().as_bytes()),
        URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes())
    );
    let sig = hmac_sha256(secret.as_bytes(), signing.as_bytes());
    let token = format!("{signing}.{}", URL_SAFE_NO_PAD.encode(sig));
    let pretty_header = serde_json::to_string_pretty(&header_v).map_err(|e| e.to_string())?;
    let pretty_payload = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    Ok(format!(
        "{token}\n\nHeader:\n{pretty_header}\n\nPayload:\n{pretty_payload}"
    ))
}

// ---------------------------------------------------------------------------
// 2. Mystery Number Identifier
// ---------------------------------------------------------------------------

fn parse_int_flexible(s: &str) -> Option<i128> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let v = if let Some(hex) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
        i128::from_str_radix(hex, 16).ok()?
    } else {
        rest.parse::<i128>().ok()?
    };
    Some(if neg { -v } else { v })
}

/// The date for a Unix-seconds value, but only when the year lands in
/// 1970–2100 — otherwise the reading is too unlikely to be worth showing.
fn unix_reading(secs: i128) -> Option<String> {
    if secs < 0 || secs > i64::MAX as i128 {
        return None;
    }
    let ts = secs as i64;
    let (y, _, _) = civil_from_days(ts.div_euclid(86400));
    if (1970..=2100).contains(&y) {
        Some(fmt_unix(ts))
    } else {
        None
    }
}

fn human_size(bytes: f64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut v = bytes;
    let mut u = 0;
    while v >= 1024.0 && u < units.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    let raw = format!("{v:.2}");
    let num = raw.trim_end_matches('0').trim_end_matches('.');
    format!("{num} {}", units[u])
}

fn number_int_readings(n: i128, lines: &mut Vec<String>) {
    if let Some(d) = unix_reading(n) {
        lines.push(format!("As Unix time (seconds): {d}"));
    }
    // Scaled readings only kick in once the value is big enough that the
    // smaller unit is plausible (at least a day's worth of the small unit).
    if n >= 86_400_000 {
        if let Some(d) = unix_reading(n / 1_000) {
            lines.push(format!("As Unix time (milliseconds): {d}"));
        }
    }
    if n >= 86_400_000_000 {
        if let Some(d) = unix_reading(n / 1_000_000) {
            lines.push(format!("As Unix time (microseconds): {d}"));
        }
    }
    // Windows FILETIME: 100-nanosecond ticks since 1601-01-01.
    if let Some(d) = unix_reading(n / 10_000_000 - 11_644_473_600) {
        lines.push(format!(
            "As Windows FILETIME (100-ns ticks since 1601): {d}"
        ));
    }
    if (20_000..=80_000).contains(&n) {
        let days = days_from_civil(1899, 12, 30) + n as i64;
        lines.push(format!("As an Excel serial date: {}", fmt_date(days)));
    }
    if (0..=u32::MAX as i128).contains(&n) {
        let v = n as u32;
        lines.push(format!(
            "As an IPv4 address: {}.{}.{}.{}",
            v >> 24,
            (v >> 16) & 0xFF,
            (v >> 8) & 0xFF,
            v & 0xFF
        ));
    }
    if n >= 1024 {
        lines.push(format!("As a file size: {}", human_size(n as f64)));
    }
    if n < 0 {
        lines.push(format!("Hexadecimal: -0x{:x}", n.unsigned_abs()));
        lines.push(format!("Binary: -0b{:b}", n.unsigned_abs()));
    } else {
        lines.push(format!("Hexadecimal: 0x{n:x}"));
        lines.push(format!("Binary: 0b{n:b}"));
    }
    if n > 0 {
        let mut tmp = n as u128;
        let mut bytes = Vec::new();
        while tmp > 0 {
            bytes.push((tmp & 0xFF) as u8);
            tmp >>= 8;
        }
        bytes.reverse();
        if bytes.iter().all(|b| (0x20..=0x7E).contains(b)) {
            let text: String = bytes.iter().map(|b| *b as char).collect();
            lines.push(format!("As ASCII text: \"{text}\""));
        }
    }
}

fn number_identify(input: &str, _param: &str) -> Result<String, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Paste a number into the Input box first.".into());
    }
    let int_val = parse_int_flexible(s);
    let float_val: Option<f64> = s.parse::<f64>().ok();
    if int_val.is_none() && float_val.is_none() {
        return Err(format!(
            "'{s}' is not a number I recognise — try a decimal integer, a 0x hex value, or a decimal fraction like 3.14."
        ));
    }
    let mut lines = vec![format!("Readings for {s}:")];
    if let Some(n) = int_val {
        number_int_readings(n, &mut lines);
    } else if let Some(f) = float_val {
        if f.fract() == 0.0 && f.abs() < 9.0e18 {
            number_int_readings(f as i128, &mut lines);
        } else {
            if (0.0..=4_102_444_800.0).contains(&f) {
                lines.push(format!("As Unix time (seconds): {}", fmt_unix(f as i64)));
            }
            lines.push(format!("As a decimal fraction: {f}"));
        }
    }
    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// 3. JSON Structural Diff
// ---------------------------------------------------------------------------

fn short_value(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 60 {
        let cut: String = s.chars().take(57).collect();
        format!("{cut}…")
    } else {
        s
    }
}

fn diff_values(a: &Value, b: &Value, path: &str, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(ma), Value::Object(mb)) => {
            for (k, va) in ma {
                let p = format!("{path}.{k}");
                match mb.get(k) {
                    None => out.push(format!("{p}: only in first")),
                    Some(vb) => diff_values(va, vb, &p, out),
                }
            }
            for k in mb.keys() {
                if !ma.contains_key(k) {
                    out.push(format!("{path}.{k}: only in second"));
                }
            }
        }
        (Value::Array(aa), Value::Array(ab)) => {
            let n = aa.len().max(ab.len());
            for i in 0..n {
                let p = format!("{path}[{i}]");
                match (aa.get(i), ab.get(i)) {
                    (Some(x), Some(y)) => diff_values(x, y, &p, out),
                    (Some(_), None) => out.push(format!("{p}: only in first")),
                    (None, Some(_)) => out.push(format!("{p}: only in second")),
                    (None, None) => {}
                }
            }
        }
        _ => {
            if a != b {
                out.push(format!("{path}: {} → {}", short_value(a), short_value(b)));
            }
        }
    }
}

fn json_diff(input: &str, param: &str) -> Result<String, String> {
    let a: Value = serde_json::from_str(input.trim())
        .map_err(|e| format!("The Input is not valid JSON: {e}"))?;
    if param.trim().is_empty() {
        return Err("Enter the second JSON document as the Parameter.".into());
    }
    let b: Value = serde_json::from_str(param.trim())
        .map_err(|e| format!("The Parameter is not valid JSON: {e}"))?;
    let mut out = Vec::new();
    diff_values(&a, &b, "$", &mut out);
    if out.is_empty() {
        Ok("The two documents are structurally identical.".into())
    } else {
        Ok(out.join("\n"))
    }
}

// ---------------------------------------------------------------------------
// 4/5. INI ⇄ JSON
// ---------------------------------------------------------------------------

/// Parse INI/properties text into a nested JSON object. Sections and dotted
/// keys nest; every value stays a string.
fn parse_ini(text: &str) -> Map<String, Value> {
    let mut root = Map::new();
    let mut section: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') && line.len() >= 2 {
            let name = line[1..line.len() - 1].trim();
            section = name
                .split('.')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            continue;
        }
        if let Some(pos) = line.find(['=', ':']) {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            if key.is_empty() {
                continue;
            }
            let mut path = section.clone();
            path.extend(
                key.split('.')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty()),
            );
            if let Some((last, parents)) = path.split_last() {
                ini_insert(&mut root, parents, last, value);
            }
        }
    }
    root
}

fn ini_insert(root: &mut Map<String, Value>, path: &[String], key: &str, value: &str) {
    let mut cur = root;
    for seg in path {
        let entry = cur
            .entry(seg.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        cur = entry.as_object_mut().expect("just made it an object");
    }
    cur.insert(key.to_string(), Value::String(value.to_string()));
}

fn ini_to_json(input: &str, _param: &str) -> Result<String, String> {
    let obj = parse_ini(input);
    if obj.is_empty() {
        return Err("No INI sections or key = value lines found in the Input.".into());
    }
    serde_json::to_string_pretty(&Value::Object(obj)).map_err(|e| e.to_string())
}

fn path_join(prefix: &[String], key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", prefix.join("."), key)
    }
}

fn ini_scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

fn ini_array(arr: &[Value], path: &str) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in arr {
        match item {
            Value::String(_) | Value::Number(_) | Value::Bool(_) => parts.push(ini_scalar(item)),
            _ => {
                return Err(format!(
                    "{path} is an array holding objects or arrays — INI can't represent those."
                ))
            }
        }
    }
    Ok(parts.join(", "))
}

fn ini_walk(
    lines: &mut Vec<String>,
    prefix: &[String],
    obj: &Map<String, Value>,
    is_root: bool,
) -> Result<(), String> {
    let mut body: Vec<String> = Vec::new();
    let mut subs: Vec<(&String, &Map<String, Value>)> = Vec::new();
    for (k, val) in obj {
        match val {
            Value::Object(m) => subs.push((k, m)),
            Value::Array(arr) => {
                body.push(format!("{k} = {}", ini_array(arr, &path_join(prefix, k))?));
            }
            Value::Null => {
                return Err(format!(
                    "{} is null — INI files have no null value.",
                    path_join(prefix, k)
                ));
            }
            _ => body.push(format!("{k} = {}", ini_scalar(val))),
        }
    }
    if !is_root {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("[{}]", prefix.join(".")));
    }
    lines.extend(body);
    for (k, m) in subs {
        let mut p2 = prefix.to_vec();
        p2.push(k.clone());
        ini_walk(lines, &p2, m, false)?;
    }
    Ok(())
}

fn json_to_ini(input: &str, _param: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(input.trim())
        .map_err(|e| format!("The Input is not valid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or("The Input must be a JSON object at the top level.")?;
    let mut lines: Vec<String> = Vec::new();
    ini_walk(&mut lines, &[], obj, true)?;
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    Ok(lines.join("\n"))
}

// ---------------------------------------------------------------------------
// 6/7/8/9. .env tools (one shared parser)
// ---------------------------------------------------------------------------

/// Parse .env text into ordered KEY=VALUE pairs: `#` comments are skipped,
/// an `export ` prefix is allowed, and quoted values are unquoted.
fn parse_env(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_string();
            if key.is_empty() {
                continue;
            }
            out.push((key, env_unquote(v.trim())));
        }
    }
    out
}

fn env_unquote(v: &str) -> String {
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        let inner = &v[1..v.len() - 1];
        let mut out = String::new();
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
    } else if v.len() >= 2 && v.starts_with('\'') && v.ends_with('\'') {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

/// Write a value back in .env form, double-quoting when it contains
/// anything that could confuse a parser.
fn env_format_value(v: &str) -> String {
    let needs_quote = v.is_empty()
        || v.chars()
            .any(|c| c.is_whitespace() || matches!(c, '#' | '"' | '\'' | '=' | '$' | '`'));
    if !needs_quote {
        return v.to_string();
    }
    let mut out = String::from("\"");
    for c in v.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn env_to_json(input: &str, _param: &str) -> Result<String, String> {
    let pairs = parse_env(input);
    if pairs.is_empty() {
        return Err("No KEY=VALUE lines found in the Input.".into());
    }
    let mut obj = Map::new();
    for (k, v) in pairs {
        obj.insert(k, Value::String(v));
    }
    serde_json::to_string_pretty(&Value::Object(obj)).map_err(|e| e.to_string())
}

fn json_to_env(input: &str, _param: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(input.trim())
        .map_err(|e| format!("The Input is not valid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or("The Input must be a flat JSON object, like {\"PORT\": 8080}.")?;
    let mut lines = Vec::new();
    for (k, val) in obj {
        match val {
            Value::String(s) => lines.push(format!("{k}={}", env_format_value(s))),
            Value::Number(n) => lines.push(format!("{k}={n}")),
            Value::Bool(b) => lines.push(format!("{k}={b}")),
            _ => {
                return Err(format!(
                    "Key '{k}' holds an object or array — .env values must be plain text, numbers or true/false."
                ))
            }
        }
    }
    Ok(lines.join("\n"))
}

fn env_diff(input: &str, param: &str) -> Result<String, String> {
    let parts = split_param(param);
    let b_text = parts.first().copied().unwrap_or("");
    if b_text.trim().is_empty() {
        return Err("Enter the second .env file's text as the Parameter (add |||mask to hide the differing values).".into());
    }
    let mask = parts
        .get(1)
        .is_some_and(|s| s.trim().eq_ignore_ascii_case("mask"));
    let a_pairs = parse_env(input);
    let b_pairs = parse_env(b_text);
    if a_pairs.is_empty() && b_pairs.is_empty() {
        return Err("Neither side has any KEY=VALUE lines.".into());
    }
    let a_map: std::collections::BTreeMap<&str, &str> = a_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let b_map: std::collections::BTreeMap<&str, &str> = b_pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let only_a: Vec<&str> = a_map
        .keys()
        .filter(|k| !b_map.contains_key(*k))
        .copied()
        .collect();
    let only_b: Vec<&str> = b_map
        .keys()
        .filter(|k| !a_map.contains_key(*k))
        .copied()
        .collect();
    let different: Vec<(&str, &str, &str)> = a_map
        .iter()
        .filter_map(|(k, av)| match b_map.get(k) {
            Some(bv) if bv != av => Some((*k, *av, *bv)),
            _ => None,
        })
        .collect();
    let mut out = vec![format!(
        "Compared the first .env ({} keys) with the second ({} keys).",
        a_map.len(),
        b_map.len()
    )];
    out.push(String::new());
    out.push(format!("Only in the first ({}):", only_a.len()));
    for k in &only_a {
        out.push(format!("  {k}"));
    }
    out.push(String::new());
    out.push(format!("Only in the second ({}):", only_b.len()));
    for k in &only_b {
        out.push(format!("  {k}"));
    }
    out.push(String::new());
    out.push(format!("Different values ({}):", different.len()));
    for (k, av, bv) in &different {
        if mask {
            out.push(format!("  {k}: (differs)"));
        } else {
            out.push(format!("  {k}: {av} → {bv}"));
        }
    }
    Ok(out.join("\n"))
}

fn env_merge(input: &str, param: &str) -> Result<String, String> {
    if param.trim().is_empty() {
        return Err(
            "Enter the second .env file's text as the Parameter — it wins on conflicts.".into(),
        );
    }
    let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for (k, v) in parse_env(input) {
        map.insert(k, v);
    }
    for (k, v) in parse_env(param) {
        map.insert(k, v);
    }
    if map.is_empty() {
        return Err("No KEY=VALUE lines found on either side.".into());
    }
    Ok(map
        .iter()
        .map(|(k, v)| format!("{k}={}", env_format_value(v)))
        .collect::<Vec<_>>()
        .join("\n"))
}

// ---------------------------------------------------------------------------
// 10. Mock Data Generator
// ---------------------------------------------------------------------------

const FIRST_NAMES: &[&str] = &[
    "Ada",
    "Grace",
    "Linus",
    "Margaret",
    "Alan",
    "Edsger",
    "Barbara",
    "Donald",
    "Radia",
    "Ken",
    "Dennis",
    "Frances",
    "Claude",
    "Katherine",
    "Tim",
    "Vint",
    "Guido",
    "Yukihiro",
    "Anders",
    "Bjarne",
    "James",
    "Brendan",
    "Ruth",
    "Sally",
];
const LAST_NAMES: &[&str] = &[
    "Lovelace",
    "Hopper",
    "Torvalds",
    "Hamilton",
    "Turing",
    "Dijkstra",
    "Liskov",
    "Knuth",
    "Perlman",
    "Thompson",
    "Ritchie",
    "Allen",
    "Shannon",
    "Johnson",
    "Berners-Lee",
    "Cerf",
    "van Rossum",
    "Matsumoto",
    "Hejlsberg",
    "Stroustrup",
    "Gosling",
    "Eich",
    "Borg",
    "Ride",
];
const CITIES: &[&str] = &[
    "Springfield",
    "Riverton",
    "Lakewood",
    "Fairview",
    "Greenville",
    "Madison",
    "Salem",
    "Clinton",
    "Georgetown",
    "Arlington",
    "Franklin",
    "Oxford",
    "Ashland",
    "Burlington",
    "Milford",
    "Auburn",
];
const WORDS: &[&str] = &[
    "apple", "bridge", "cloud", "delta", "ember", "falcon", "garden", "harbor", "island", "jungle",
    "kitten", "ladder", "meadow", "needle", "ocean", "piano", "quartz", "river", "saddle",
    "temple", "umbrella", "valley", "window", "xenon", "yellow", "zephyr", "copper", "velvet",
    "marble", "willow", "cinder", "pluto",
];
const COMPANY_SUFFIXES: &[&str] = &[
    "Labs",
    "Systems",
    "Works",
    "Group",
    "Industries",
    "Softworks",
    "Technologies",
    "Solutions",
];
const EMAIL_DOMAINS: &[&str] = &[
    "example.com",
    "mail.test",
    "demo.org",
    "sample.net",
    "corp.example",
];

fn pick<'a>(rng: &mut Rng, items: &'a [&'a str]) -> &'a str {
    items[rng.below(items.len() as u64) as usize]
}

/// A random v4 UUID from the project RNG (mirrors hashgen.rs).
fn uuid_from(rng: &mut Rng) -> String {
    let mut b = [0u8; 16];
    for chunk in b.chunks_mut(8) {
        let v = rng.next_u64().to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15])
}

fn placeholder_value(ph: &str, rng: &mut Rng) -> Result<String, String> {
    let (name, arg) = match ph.split_once(':') {
        Some((n, a)) => (n.trim(), Some(a.trim())),
        None => (ph, None),
    };
    match name {
        "uuid" => Ok(uuid_from(rng)),
        "firstName" => Ok(pick(rng, FIRST_NAMES).to_string()),
        "lastName" => Ok(pick(rng, LAST_NAMES).to_string()),
        "fullName" | "name" => Ok(format!(
            "{} {}",
            pick(rng, FIRST_NAMES),
            pick(rng, LAST_NAMES)
        )),
        "email" => Ok(format!(
            "{}.{}@{}",
            pick(rng, FIRST_NAMES).to_lowercase(),
            pick(rng, LAST_NAMES).to_lowercase().replace(' ', ""),
            pick(rng, EMAIL_DOMAINS)
        )),
        "int" => {
            let a = arg.ok_or("{{int}} needs a range, like {{int:1:100}}.")?;
            let (lo, hi) = a
                .split_once(':')
                .ok_or("{{int}} needs a range, like {{int:1:100}}.")?;
            let lo: i128 = lo
                .trim()
                .parse()
                .map_err(|_| format!("'{lo}' is not a whole number in the int range."))?;
            let hi: i128 = hi
                .trim()
                .parse()
                .map_err(|_| format!("'{hi}' is not a whole number in the int range."))?;
            if hi < lo {
                return Err(format!(
                    "The int range {lo}:{hi} is backwards — the low end must come first."
                ));
            }
            let span = hi - lo;
            if span > u64::MAX as i128 - 1 {
                return Err("That int range is too wide to pick from.".into());
            }
            Ok(format!("{}", lo + rng.below((span + 1) as u64) as i128))
        }
        "float" => {
            let a = arg.ok_or("{{float}} needs a range, like {{float:0:9.99}}.")?;
            let (lo, hi) = a
                .split_once(':')
                .ok_or("{{float}} needs a range, like {{float:0:9.99}}.")?;
            let lo: f64 = lo
                .trim()
                .parse()
                .map_err(|_| format!("'{lo}' is not a number in the float range."))?;
            let hi: f64 = hi
                .trim()
                .parse()
                .map_err(|_| format!("'{hi}' is not a number in the float range."))?;
            if hi < lo {
                return Err(format!(
                    "The float range {lo}:{hi} is backwards — the low end must come first."
                ));
            }
            let v = lo + rng.f64() * (hi - lo);
            let raw = format!("{v:.4}");
            Ok(raw.trim_end_matches('0').trim_end_matches('.').to_string())
        }
        "bool" => Ok(if rng.below(2) == 0 {
            "true".to_string()
        } else {
            "false".to_string()
        }),
        "word" => Ok(pick(rng, WORDS).to_string()),
        "city" => Ok(pick(rng, CITIES).to_string()),
        "company" => Ok(format!(
            "{} {}",
            pick(rng, LAST_NAMES),
            pick(rng, COMPANY_SUFFIXES)
        )),
        "url" => Ok(format!(
            "https://www.{}.{}",
            pick(rng, WORDS),
            pick(rng, &["com", "io", "dev", "net", "org"])
        )),
        "ipv4" => Ok(format!(
            "{}.{}.{}.{}",
            1 + rng.below(255),
            rng.below(256),
            rng.below(256),
            rng.below(256)
        )),
        "color" => Ok(format!("#{:06X}", rng.below(0x100_0000))),
        "date" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(1_700_000_000);
            let back = rng.below(3 * 365 * 24 * 3600) as i64;
            Ok(fmt_unix(now - back)[..10].to_string())
        }
        "choice" => {
            let a = arg.ok_or("{{choice}} needs options, like {{choice:a|b|c}}.")?;
            let options: Vec<&str> = a.split('|').collect();
            if options.is_empty() {
                return Err("{{choice}} needs at least one option.".into());
            }
            Ok(options[rng.below(options.len() as u64) as usize].to_string())
        }
        _ => Err(format!("Unknown placeholder: {ph}")),
    }
}

fn fill_placeholders(s: &str, rng: &mut Rng) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or_else(|| format!("A placeholder in '{s}' is never closed with '}}}}'."))?;
        let ph = after[..end].trim();
        out.push_str(&placeholder_value(ph, rng)?);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

fn fill_template(v: &Value, rng: &mut Rng) -> Result<Value, String> {
    match v {
        Value::String(s) => Ok(Value::String(fill_placeholders(s, rng)?)),
        Value::Array(a) => {
            let mut out = Vec::new();
            for item in a {
                out.push(fill_template(item, rng)?);
            }
            Ok(Value::Array(out))
        }
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, item) in m {
                out.insert(k.clone(), fill_template(item, rng)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

fn mock_data(input: &str, param: &str) -> Result<String, String> {
    let template: Value = serde_json::from_str(input.trim())
        .map_err(|e| format!("The template is not valid JSON: {e}"))?;
    if !template.is_object() && !template.is_array() {
        return Err("The template must be a JSON object or array.".into());
    }
    let p = param.trim();
    let count: usize = if p.is_empty() {
        1
    } else {
        p.parse::<usize>()
            .map_err(|_| format!("'{p}' is not a whole number for the row count."))?
    };
    if count == 0 {
        return Err("The row count must be at least 1.".into());
    }
    let count = count.min(200);
    let mut rng = Rng::from_time();
    let mut rows = Vec::new();
    for _ in 0..count {
        rows.push(fill_template(&template, &mut rng)?);
    }
    serde_json::to_string_pretty(&Value::Array(rows)).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 11/12/13. SQL helpers
// ---------------------------------------------------------------------------

fn sql_in_list(input: &str, param: &str) -> Result<String, String> {
    let values: Vec<&str> = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if values.is_empty() {
        return Err("Enter one value per line in the Input box.".into());
    }
    let numbers = param.trim().eq_ignore_ascii_case("numbers");
    let mut parts = Vec::new();
    for v in values {
        if numbers {
            if v.parse::<f64>().is_err() {
                return Err(format!(
                    "'{v}' is not a number, but the Parameter asks for numbers mode."
                ));
            }
            parts.push(v.to_string());
        } else {
            parts.push(format!("'{}'", v.replace('\'', "''")));
        }
    }
    Ok(format!("({})", parts.join(", ")))
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// A deliberately small CSV row parser: quotes group fields, "" is an
/// escaped quote, and fields can't span lines.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' && cur.is_empty() {
            in_quotes = true;
        } else if c == ',' {
            fields.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    fields.push(cur);
    fields
}

fn data_from_json(text: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| {
        format!("The Input starts with '[' so it was read as JSON, but it isn't valid: {e}")
    })?;
    let arr = v
        .as_array()
        .ok_or("The JSON must be an array of objects.")?;
    let mut headers: Vec<String> = Vec::new();
    for item in arr {
        let obj = item
            .as_object()
            .ok_or("Every item in the JSON array must be an object.")?;
        for k in obj.keys() {
            if !headers.contains(k) {
                headers.push(k.clone());
            }
        }
    }
    let mut rows = Vec::new();
    for item in arr {
        if let Some(obj) = item.as_object() {
            rows.push(
                headers
                    .iter()
                    .map(|h| match obj.get(h) {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Null) | None => String::new(),
                        Some(other) => other.to_string(),
                    })
                    .collect(),
            );
        }
    }
    Ok((headers, rows))
}

fn data_from_csv(text: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut lines = text.lines();
    let headers = match lines.next() {
        Some(h) => parse_csv_row(h),
        None => return Err("The CSV is empty.".into()),
    };
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut row = parse_csv_row(line);
        row.resize(headers.len(), String::new());
        rows.push(row);
    }
    Ok((headers, rows))
}

fn infer_column_type(rows: &[Vec<String>], col: usize) -> &'static str {
    let mut any = false;
    let mut all_int = true;
    let mut all_num = true;
    let mut all_boolish = true;
    let mut any_bool_word = false;
    for row in rows {
        let v = row.get(col).map(|s| s.trim()).unwrap_or("");
        if v.is_empty() {
            continue;
        }
        any = true;
        if v.parse::<i64>().is_err() {
            all_int = false;
        }
        if v.parse::<f64>().is_err() {
            all_num = false;
        }
        let low = v.to_ascii_lowercase();
        if low == "true" || low == "false" {
            any_bool_word = true;
        } else if v != "0" && v != "1" {
            all_boolish = false;
        }
    }
    if !any {
        "TEXT"
    } else if all_boolish && any_bool_word {
        "BOOLEAN"
    } else if all_int {
        "INTEGER"
    } else if all_num {
        "REAL"
    } else {
        "TEXT"
    }
}

fn sql_create_table(input: &str, param: &str) -> Result<String, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err(
            "Paste CSV data (first row = headers) or a JSON array of objects into the Input box."
                .into(),
        );
    }
    let table = if param.trim().is_empty() {
        "my_table"
    } else {
        param.trim()
    };
    let (headers, rows) = if text.starts_with('[') {
        data_from_json(text)?
    } else {
        data_from_csv(text)?
    };
    if headers.is_empty() {
        return Err("No columns found in the data.".into());
    }
    let mut out = vec![format!("CREATE TABLE {} (", quote_ident(table))];
    for (i, h) in headers.iter().enumerate() {
        let comma = if i + 1 < headers.len() { "," } else { "" };
        out.push(format!(
            "    {} {}{}",
            quote_ident(h),
            infer_column_type(&rows, i),
            comma
        ));
    }
    out.push(");".to_string());
    Ok(out.join("\n"))
}

fn sql_literal(v: &str) -> String {
    if v.eq_ignore_ascii_case("null") {
        "NULL".to_string()
    } else if v.parse::<i64>().is_ok() || v.parse::<f64>().is_ok() {
        v.to_string()
    } else {
        format!("'{}'", v.replace('\'', "''"))
    }
}

fn sql_expand(input: &str, param: &str) -> Result<String, String> {
    // Find the ? placeholders, skipping any inside single-quoted strings.
    let chars: Vec<char> = input.chars().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut in_str = false;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\'' if in_str => {
                if chars.get(i + 1) == Some(&'\'') {
                    i += 1; // '' is an escaped quote — stay in the string
                } else {
                    in_str = false;
                }
            }
            '\'' => in_str = true,
            '?' if !in_str => positions.push(i),
            _ => {}
        }
        i += 1;
    }
    let values = split_param(param);
    if positions.len() != values.len() {
        return Err(format!(
            "The SQL has {} ? placeholder(s) but {} value(s) were given in the Parameter (separate values with |||).",
            positions.len(),
            values.len()
        ));
    }
    let mut out = String::new();
    let mut last = 0usize;
    for (vi, pos) in positions.iter().enumerate() {
        out.extend(chars[last..*pos].iter().copied());
        out.push_str(&sql_literal(values[vi].trim()));
        last = pos + 1;
    }
    out.extend(chars[last..].iter().copied());
    Ok(out)
}

// ---------------------------------------------------------------------------
// 14/15/16. Git helpers
// ---------------------------------------------------------------------------

const GITIGNORE_NAMES: &[&str] = &[
    "rust",
    "node",
    "python",
    "java",
    "go",
    "csharp",
    "vscode",
    "jetbrains",
    "windows",
    "macos",
    "linux",
];

fn gitignore_template(name: &str) -> Option<&'static str> {
    let tpl = match name {
        "rust" => "target/\n**/*.rs.bk\n*.pdb",
        "node" => {
            "node_modules/\n*.log\nnpm-debug.log*\nyarn-debug.log*\nyarn-error.log*\ncoverage/\ndist/\n.next/\n.nuxt/\n.cache/\n.env.local"
        }
        "python" => {
            "__pycache__/\n*.py[cod]\n*$py.class\n.venv/\nvenv/\nenv/\n.pytest_cache/\n.mypy_cache/\n*.egg-info/\ndist/\nbuild/\n.env"
        }
        "java" => "*.class\n*.jar\n*.war\n*.ear\ntarget/\nbuild/\n.gradle/\n*.log\nhs_err_pid*",
        "go" => "*.exe\n*.exe~\n*.dll\n*.so\n*.dylib\n*.test\n*.out\ncoverage.txt",
        "csharp" => {
            "[Bb]in/\n[Oo]bj/\n[Dd]ebug/\n[Rr]elease/\n.vs/\n*.user\n*.suo\n*.userosscache\npackages/"
        }
        "vscode" => {
            ".vscode/*\n!.vscode/settings.json\n!.vscode/tasks.json\n!.vscode/launch.json\n!.vscode/extensions.json\n*.code-workspace"
        }
        "jetbrains" => ".idea/\n*.iml\n*.iws\n*.ipr\nout/\n.idea_modules/",
        "windows" => {
            "Thumbs.db\nehthumbs.db\nDesktop.ini\n$RECYCLE.BIN/\n*.lnk\n[Dd]esktop.ini"
        }
        "macos" => ".DS_Store\n.AppleDouble\n.LSOverride\n._*\n.Spotlight-V100\n.Trashes",
        "linux" => "*~\n.fuse_hidden*\n.directory\n.Trash-*\n.nfs*",
        _ => return None,
    };
    Some(tpl)
}

fn gitignore_gen(_input: &str, param: &str) -> Result<String, String> {
    let names: Vec<String> = param
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .collect();
    if names.is_empty() {
        return Err(format!(
            "Enter one or more names as the Parameter. Valid names: {}.",
            GITIGNORE_NAMES.join(", ")
        ));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for name in &names {
        if seen.contains(name) {
            continue;
        }
        seen.push(name.clone());
        match gitignore_template(name) {
            Some(tpl) => out.push(format!("# ===== {name} =====\n{}", tpl.trim_end())),
            None => {
                return Err(format!(
                    "'{name}' isn't a known template. Valid names: {}.",
                    GITIGNORE_NAMES.join(", ")
                ))
            }
        }
    }
    Ok(out.join("\n\n"))
}

const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

fn commit_check(input: &str, _param: &str) -> Result<String, String> {
    if input.trim().is_empty() {
        return Err("Paste a commit message into the Input box.".into());
    }
    let all_lines: Vec<&str> = input.trim_end_matches('\n').lines().collect();
    let header = all_lines.first().copied().unwrap_or("");
    let mut checks: Vec<String> = Vec::new();
    let mut issues = 0usize;

    let (prefix, subject) = match header.split_once(": ") {
        Some((p, s)) => (Some(p), s),
        None => (None, header),
    };
    match prefix {
        Some(p) => {
            let base = p.strip_suffix('!').unwrap_or(p);
            let ty = base.split('(').next().unwrap_or("");
            if COMMIT_TYPES.contains(&ty) {
                checks.push(format!(
                    "✓ The type '{ty}' is a conventional type and the shape is right"
                ));
            } else {
                issues += 1;
                checks.push(format!(
                    "✗ '{ty}' isn't a conventional type — use one of: {}",
                    COMMIT_TYPES.join(", ")
                ));
            }
            if base.contains('(') && !base.ends_with(')') {
                issues += 1;
                checks.push("✗ The scope in (parentheses) is never closed".to_string());
            }
        }
        None => {
            issues += 1;
            checks.push(
                "✗ The first line doesn't start with a type and ': ' — expected something like 'feat(parser): add a thing'"
                    .to_string(),
            );
        }
    }
    let subj_len = subject.chars().count();
    if subject.trim().is_empty() {
        issues += 1;
        checks.push("✗ The subject (the part after ': ') is empty".to_string());
    } else if subj_len <= 72 {
        checks.push(format!("✓ Subject is {subj_len} characters (limit 72)"));
    } else {
        issues += 1;
        checks.push(format!(
            "✗ Subject is {subj_len} characters — over the 72-character limit"
        ));
    }
    if subject.ends_with('.') {
        issues += 1;
        checks.push("✗ Subject ends with a full stop — drop it".to_string());
    } else if !subject.trim().is_empty() {
        checks.push("✓ Subject has no trailing full stop".to_string());
    }
    if all_lines.len() > 1 {
        if all_lines[1].trim().is_empty() {
            checks.push("✓ Body is separated from the subject by a blank line".to_string());
        } else {
            issues += 1;
            checks.push(
                "✗ The body starts right after the subject — leave a blank line between them"
                    .to_string(),
            );
        }
    }
    if issues == 0 {
        checks.push("Overall: looks good ✓".to_string());
    } else {
        checks.push(format!("Overall: {issues} issue(s) to fix."));
    }
    Ok(checks.join("\n"))
}

/// Strip a leading `git log` hash (7–40 hex chars followed by whitespace).
fn strip_hash(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut end = 0;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    let first = &line[..end];
    if end < line.len()
        && (7..=40).contains(&first.len())
        && first.chars().all(|c| c.is_ascii_hexdigit())
    {
        line[end..].trim_start()
    } else {
        line
    }
}

/// Which changelog group a line belongs in, and how its bullet should read.
fn classify_commit(line: &str) -> (usize, String) {
    if let Some((p, s)) = line.split_once(": ") {
        let base = p.strip_suffix('!').unwrap_or(p);
        let (ty, scope) = match base.split_once('(') {
            Some((t, sc)) if sc.ends_with(')') => (t, Some(sc[..sc.len() - 1].to_string())),
            _ => (base, None),
        };
        let group = match ty {
            "feat" => 0,
            "fix" => 1,
            "perf" => 2,
            "refactor" => 3,
            "docs" => 4,
            _ => 5,
        };
        let bullet = match scope {
            Some(sc) if !sc.is_empty() => format!("{s} ({sc})"),
            _ => s.to_string(),
        };
        (group, bullet)
    } else {
        (5, line.to_string())
    }
}

fn changelog_from_commits(input: &str, _param: &str) -> Result<String, String> {
    if input.trim().is_empty() {
        return Err("Paste one commit message per line into the Input box.".into());
    }
    let mut groups: Vec<(&str, Vec<String>)> = vec![
        ("Features", Vec::new()),
        ("Fixes", Vec::new()),
        ("Performance", Vec::new()),
        ("Refactoring", Vec::new()),
        ("Documentation", Vec::new()),
        ("Other Changes", Vec::new()),
    ];
    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let (group, bullet) = classify_commit(strip_hash(line));
        groups[group].1.push(bullet);
    }
    let mut out: Vec<String> = Vec::new();
    for (name, items) in &groups {
        if items.is_empty() {
            continue;
        }
        out.push(format!("## {name}"));
        for b in items {
            out.push(format!("- {b}"));
        }
        out.push(String::new());
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    Ok(out.join("\n"))
}

// ---------------------------------------------------------------------------
// 17. Regex Explainer
// ---------------------------------------------------------------------------

/// A small best-effort regex tokenizer: it walks the pattern element by
/// element and writes one plain-English line per element. Lines inside
/// groups are indented by depth, and output stops at about 200 lines.
struct Explainer {
    chars: Vec<char>,
    pos: usize,
    lines: Vec<String>,
    groups: usize,
    truncated: bool,
}

impl Explainer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn emit(&mut self, depth: usize, text: String) {
        if self.lines.len() >= 200 {
            if !self.truncated {
                self.truncated = true;
                self.lines
                    .push("… (explanation stopped at 200 lines)".to_string());
            }
            return;
        }
        self.lines.push(format!("{}{}", "  ".repeat(depth), text));
    }

    fn run(&mut self, depth: usize, in_group: bool) {
        while let Some(c) = self.peek() {
            if self.truncated {
                return;
            }
            if c == ')' && in_group {
                return; // the group() caller consumes the closing paren
            }
            self.element(depth);
        }
    }

    fn element(&mut self, depth: usize) {
        match self.peek() {
            Some('\\') => {
                self.bump();
                self.escape(depth);
                self.quantifier(depth);
            }
            Some('[') => {
                self.bump();
                let desc = self.char_class();
                self.emit(depth, desc);
                self.quantifier(depth);
            }
            Some('(') => {
                self.group(depth);
            }
            Some('|') => {
                self.bump();
                self.emit(
                    depth,
                    "— or — (alternation: either side of the | can match)".to_string(),
                );
            }
            Some('^') => {
                self.bump();
                self.emit(
                    depth,
                    "start of the text (or of a line, in multi-line mode)".to_string(),
                );
            }
            Some('$') => {
                self.bump();
                self.emit(
                    depth,
                    "end of the text (or of a line, in multi-line mode)".to_string(),
                );
            }
            Some('.') => {
                self.bump();
                self.emit(depth, "any character except a newline".to_string());
                self.quantifier(depth);
            }
            Some(')') => {
                self.bump();
                self.emit(depth, "an unmatched ')' — passed through".to_string());
            }
            Some(c) if "*+?".contains(c) => {
                self.bump();
                self.emit(
                    depth,
                    format!("a quantifier ('{c}') with nothing before it — usually a mistake"),
                );
            }
            Some('{') => {
                self.bump();
                self.emit(
                    depth,
                    "the character '{' (it doesn't form a valid {n} or {n,m} quantifier here)"
                        .to_string(),
                );
            }
            Some(c) => {
                self.bump();
                self.emit(depth, format!("the character '{c}'"));
                self.quantifier(depth);
            }
            None => {}
        }
    }

    fn escape(&mut self, depth: usize) {
        match self.bump() {
            None => {
                self.emit(
                    depth,
                    "a trailing backslash with nothing after it".to_string(),
                );
            }
            Some(e) => {
                let desc = match e {
                    'd' => "a digit (0–9)".to_string(),
                    'D' => "any character that is NOT a digit".to_string(),
                    'w' => "a word character (letter, digit or underscore)".to_string(),
                    'W' => "any character that is NOT a word character".to_string(),
                    's' => "a whitespace character (space, tab, newline…)".to_string(),
                    'S' => "any character that is NOT whitespace".to_string(),
                    'b' => {
                        "a word boundary (the edge between a word character and a non-word character)"
                            .to_string()
                    }
                    'B' => "a non-word-boundary (NOT the edge of a word)".to_string(),
                    'n' => "a newline".to_string(),
                    't' => "a tab".to_string(),
                    'r' => "a carriage return".to_string(),
                    '0' => "a null character".to_string(),
                    '1'..='9' => {
                        format!("a backreference — the same text that group {e} matched")
                    }
                    c if c.is_alphanumeric() => {
                        format!("unrecognized escape \\{c} — passed through")
                    }
                    c => format!("a literal '{c}' (the backslash escapes it)"),
                };
                self.emit(depth, desc);
            }
        }
    }

    /// Parse a `[...]` class; the opening bracket is already consumed.
    fn char_class(&mut self) -> String {
        let negated = if self.peek() == Some('^') {
            self.bump();
            true
        } else {
            false
        };
        let mut items: Vec<String> = Vec::new();
        let mut first = true;
        while let Some(c) = self.peek() {
            if c == ']' && !first {
                self.bump();
                break;
            }
            first = false;
            if c == '\\' {
                self.bump();
                match self.bump() {
                    Some('d') => items.push("a digit".to_string()),
                    Some('D') => items.push("a non-digit".to_string()),
                    Some('w') => items.push("a word character".to_string()),
                    Some('W') => items.push("a non-word character".to_string()),
                    Some('s') => items.push("whitespace".to_string()),
                    Some('S') => items.push("non-whitespace".to_string()),
                    Some('n') => items.push("a newline".to_string()),
                    Some('t') => items.push("a tab".to_string()),
                    Some(other) => items.push(format!("'{other}'")),
                    None => items.push("'\\'".to_string()),
                }
                continue;
            }
            // A range like a-z (but a trailing '-' is a literal).
            if self.peek2() == Some('-')
                && self
                    .chars
                    .get(self.pos + 2)
                    .copied()
                    .is_some_and(|n| n != ']')
            {
                let lo = c;
                self.bump(); // the low end
                self.bump(); // the '-'
                let hi = self.bump().unwrap_or('-');
                items.push(format!("'{lo}' to '{hi}'"));
                continue;
            }
            self.bump();
            items.push(format!("'{c}'"));
        }
        let body = if items.is_empty() {
            "nothing (an empty class)".to_string()
        } else {
            items.join(", ")
        };
        if negated {
            format!("a character class: any character EXCEPT {body}")
        } else {
            format!("a character class: any of {body}")
        }
    }

    /// If the element just explained is followed by a quantifier, explain
    /// that too. A `{` that doesn't form a valid {n}/{n,}/{n,m} is left
    /// alone so it can be explained as a literal character.
    fn quantifier(&mut self, depth: usize) {
        let desc: Option<String> = match self.peek() {
            Some('*') => {
                self.bump();
                Some("zero or more times".to_string())
            }
            Some('+') => {
                self.bump();
                Some("one or more times".to_string())
            }
            Some('?') => {
                self.bump();
                Some("zero or one time (optional)".to_string())
            }
            Some('{') => {
                let save = self.pos;
                self.bump();
                let mut inside = String::new();
                let mut valid = false;
                while let Some(c) = self.peek() {
                    if c == '}' {
                        valid = true;
                        self.bump();
                        break;
                    }
                    if !(c.is_ascii_digit() || c == ',') {
                        break;
                    }
                    inside.push(c);
                    self.bump();
                }
                if valid && !inside.is_empty() {
                    Some(match inside.split_once(',') {
                        Some((a, "")) => format!("{a} or more times"),
                        Some(("", b)) => format!("at most {b} times"),
                        Some((a, b)) => format!("between {a} and {b} times"),
                        None => format!("exactly {inside} times"),
                    })
                } else {
                    self.pos = save;
                    None
                }
            }
            _ => None,
        };
        if let Some(mut d) = desc {
            if self.peek() == Some('?') {
                self.bump();
                d.push_str(" — lazy: as few as possible");
            }
            self.emit(depth, format!("↳ repeated {d}"));
        }
    }

    /// Parse a `(...)` group; the opening paren is the current character.
    fn group(&mut self, depth: usize) {
        self.bump(); // consume '('
        let kind: Option<String>;
        if self.peek() == Some('?') {
            let rest: String = self.chars[self.pos..].iter().collect();
            if rest.starts_with("?:") {
                self.pos += 2;
                kind = Some("non-capturing group (groups things without numbering them)".into());
            } else if rest.starts_with("?=") {
                self.pos += 2;
                kind =
                    Some("lookahead group (the text ahead must match, but isn't consumed)".into());
            } else if rest.starts_with("?!") {
                self.pos += 2;
                kind = Some("negative lookahead group (the text ahead must NOT match)".into());
            } else if rest.starts_with("?<=") {
                self.pos += 3;
                kind = Some("lookbehind group (the text behind must match)".into());
            } else if rest.starts_with("?<!") {
                self.pos += 3;
                kind = Some("negative lookbehind group (the text behind must NOT match)".into());
            } else if rest.starts_with("?<") || rest.starts_with("?P<") {
                let name_start = if rest.starts_with("?P<") { 3 } else { 2 };
                let after: String = self.chars[self.pos + name_start..].iter().collect();
                match after.find('>') {
                    Some(end) => {
                        let name: String = after.chars().take(end).collect();
                        self.pos += name_start + end + 1;
                        self.groups += 1;
                        kind = Some(format!(
                            "named capturing group (name: {name}) — this is group {}",
                            self.groups
                        ));
                    }
                    None => {
                        kind = Some("a group whose name is never closed with '>'".into());
                    }
                }
            } else {
                // Maybe inline flags: (?i), (?im-s), …
                let mut j = self.pos + 1;
                let mut flags = String::new();
                while let Some(c) = self.chars.get(j).copied() {
                    if c.is_ascii_alphabetic() || c == '-' {
                        flags.push(c);
                        j += 1;
                    } else {
                        break;
                    }
                }
                if !flags.is_empty() && self.chars.get(j) == Some(&')') {
                    self.pos = j + 1;
                    self.emit(
                        depth,
                        format!("pattern flags ({flags}) — i: case-insensitive, m: multi-line, s: dot matches newlines too, x: ignore whitespace in the pattern"),
                    );
                    return;
                }
                kind = Some("a group with an unrecognized '(?' prefix".into());
            }
        } else {
            self.groups += 1;
            kind = Some(format!(
                "capturing group {} (its match is remembered for later use)",
                self.groups
            ));
        }
        let kind = kind.unwrap_or_else(|| "group".to_string());
        self.emit(depth, format!("start of {kind}:"));
        self.run(depth + 1, true);
        if self.peek() == Some(')') {
            self.bump();
        }
        self.emit(depth, "end of group".to_string());
        self.quantifier(depth);
    }
}

fn regex_explain(input: &str, _param: &str) -> Result<String, String> {
    let pattern = input.trim();
    if pattern.is_empty() {
        return Err("Paste a regular expression pattern into the Input box.".into());
    }
    let mut ex = Explainer {
        chars: pattern.chars().collect(),
        pos: 0,
        lines: Vec::new(),
        groups: 0,
        truncated: false,
    };
    ex.run(0, false);
    if ex.lines.is_empty() {
        return Ok("Nothing to explain.".into());
    }
    Ok(ex.lines.join("\n"))
}

// ---------------------------------------------------------------------------
// 18. Log File Analyzer
// ---------------------------------------------------------------------------

fn log_analyze(input: &str, _param: &str) -> Result<String, String> {
    if input.trim().is_empty() {
        return Err("Paste or drop log text into the Input box.".into());
    }
    let lines: Vec<&str> = input.lines().collect();
    let (mut err, mut warn, mut info, mut debug, mut trace) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for line in &lines {
        let up = line.to_uppercase();
        if up.contains("FATAL") || up.contains("ERROR") || up.contains("ERR") {
            err += 1;
        } else if up.contains("WARN") {
            warn += 1;
        } else if up.contains("INFO") {
            info += 1;
        } else if up.contains("DEBUG") {
            debug += 1;
        } else if up.contains("TRACE") {
            trace += 1;
        }
    }
    let mut busiest = "none (no level words found)";
    let mut best = 0usize;
    for (name, n) in [
        ("ERROR", err),
        ("WARN", warn),
        ("INFO", info),
        ("DEBUG", debug),
        ("TRACE", trace),
    ] {
        if n > best {
            best = n;
            busiest = name;
        }
    }
    let busiest_text = if best == 0 {
        busiest.to_string()
    } else {
        format!("{busiest} ({best} lines)")
    };

    let ts_re =
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}").expect("static pattern");
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    for m in ts_re.find_iter(input) {
        if first_ts.is_none() {
            first_ts = Some(m.as_str().to_string());
        }
        last_ts = Some(m.as_str().to_string());
    }

    // Most repeated lines, with hex addresses and digit runs normalised so
    // the same message at different times/addresses counts as one line.
    let hex_re = regex::Regex::new(r"0x[0-9a-fA-F]+").expect("static pattern");
    let digit_re = regex::Regex::new(r"\d+").expect("static pattern");
    let mut counts: Vec<(String, usize, usize)> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let no_hex = hex_re.replace_all(line, "0x#");
        let norm = digit_re.replace_all(&no_hex, "#").trim().to_string();
        if norm.is_empty() {
            continue;
        }
        if let Some(entry) = counts.iter_mut().find(|(n, _, _)| *n == norm) {
            entry.1 += 1;
        } else {
            counts.push((norm, 1, idx));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

    let mut out = vec![
        format!("Total lines: {}", lines.len()),
        format!("ERROR (including FATAL and ERR): {err}"),
        format!("WARN: {warn}"),
        format!("INFO: {info}"),
        format!("DEBUG: {debug}"),
        format!("TRACE: {trace}"),
        format!("Busiest level: {busiest_text}"),
    ];
    if let (Some(f), Some(l)) = (&first_ts, &last_ts) {
        out.push(String::new());
        out.push(format!("First timestamp: {f}"));
        out.push(format!("Last timestamp: {l}"));
    }
    let repeated: Vec<&(String, usize, usize)> =
        counts.iter().filter(|(_, n, _)| *n >= 2).take(8).collect();
    if !repeated.is_empty() {
        out.push(String::new());
        out.push("Most repeated lines (numbers replaced with #, hex addresses with 0x#):".into());
        for (text, n, _) in repeated {
            out.push(format!("  {n} × {text}"));
        }
    }
    Ok(out.join("\n"))
}

// ---------------------------------------------------------------------------
// 19. Stack Trace Cleaner
// ---------------------------------------------------------------------------

/// Lines like `#12 0x00007f...` count as frames too.
fn is_hash_frame(trimmed: &str) -> bool {
    match trimmed.strip_prefix('#') {
        Some(rest) => rest.chars().take_while(|c| c.is_ascii_digit()).count() > 0,
        None => false,
    }
}

/// Shorten a path of 4+ segments to `…/` plus its last two segments.
fn shorten_path(path: &str) -> String {
    let segs: Vec<&str> = path.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    if segs.len() < 4 {
        return path.to_string();
    }
    format!("…/{}", segs[segs.len() - 2..].join("/"))
}

fn stack_clean(input: &str, _param: &str) -> Result<String, String> {
    if input.trim().is_empty() {
        return Err("Paste a stack trace into the Input box.".into());
    }
    let path_re = regex::Regex::new(r#"(?:[^\s"'()\[\]{}<>]+[/\\]){3,}[^\s"'()\[\]{}<>]+"#)
        .expect("static pattern");
    let mut frames = 0usize;
    let mut cleaned: Vec<String> = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("at ") || trimmed.starts_with("File \"") || is_hash_frame(trimmed) {
            frames += 1;
        }
        let shortened = path_re.replace_all(line, |caps: &regex::Captures| shorten_path(&caps[0]));
        cleaned.push(shortened.into_owned());
    }
    // Fold runs of identical lines into one line with a ×N count.
    let mut runs: Vec<(String, usize)> = Vec::new();
    for line in cleaned {
        match runs.last_mut() {
            Some((prev, n)) if *prev == line => *n += 1,
            _ => runs.push((line, 1)),
        }
    }
    let mut out: Vec<String> = Vec::new();
    for (line, n) in &runs {
        if *n > 1 {
            out.push(format!("{line}  ×{n}"));
        } else {
            out.push(line.clone());
        }
    }
    out.push(format!(
        "— {frames} frames, {} lines after cleaning",
        out.len()
    ));
    Ok(out.join("\n"))
}

// ---------------------------------------------------------------------------
// 20. Mojibake Fixer (UTF-8 read as Latin-1/Windows-1252)
// ---------------------------------------------------------------------------

/// The two-character sequences that give mojibake away: a UTF-8 lead byte
/// (Ã, Â, â, ð) that was mis-decoded as Latin-1/Windows-1252, followed by
/// what was a continuation byte.
fn mojibake_markers(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for w in chars.windows(2) {
        let (a, b) = (w[0], w[1]);
        let cont = ('\u{80}'..='\u{BF}').contains(&b);
        if (a == 'Ã' && cont)
            || (a == 'Â' && (cont || b == '\u{A0}'))
            || (a == 'â' && (b == '€' || b == '\u{201A}' || b == '\u{201E}'))
            || (a == 'ð' && b == 'Ÿ')
        {
            return true;
        }
    }
    false
}

/// Map a character back to the single byte it came from when UTF-8 bytes
/// were decoded as Latin-1/Windows-1252.
fn char_to_latin_byte(c: char) -> Option<u8> {
    match c {
        '\u{0}'..='\u{FF}' => Some(c as u8),
        // The Windows-1252 printable characters in the 0x80–0x9F byte range.
        '\u{20AC}' => Some(0x80), // €
        '\u{201A}' => Some(0x82), // ‚
        '\u{0192}' => Some(0x83), // ƒ
        '\u{201E}' => Some(0x84), // „
        '\u{2026}' => Some(0x85), // …
        '\u{2020}' => Some(0x86), // †
        '\u{2021}' => Some(0x87), // ‡
        '\u{02C6}' => Some(0x88), // ˆ
        '\u{2030}' => Some(0x89), // ‰
        '\u{0160}' => Some(0x8A), // Š
        '\u{2039}' => Some(0x8B), // ‹
        '\u{0152}' => Some(0x8C), // Œ
        '\u{017D}' => Some(0x8E), // Ž
        '\u{2018}' => Some(0x91), // '
        '\u{2019}' => Some(0x92), // '
        '\u{201C}' => Some(0x93), // "
        '\u{201D}' => Some(0x94), // "
        '\u{2022}' => Some(0x95), // •
        '\u{2013}' => Some(0x96), // –
        '\u{2014}' => Some(0x97), // —
        '\u{02DC}' => Some(0x98), // ˜
        '\u{2122}' => Some(0x99), // ™
        '\u{0161}' => Some(0x9A), // š
        '\u{203A}' => Some(0x9B), // ›
        '\u{0153}' => Some(0x9C), // œ
        '\u{017E}' => Some(0x9E), // ž
        '\u{0178}' => Some(0x9F), // Ÿ
        _ => None,
    }
}

fn mojibake_fix(input: &str, _param: &str) -> Result<String, String> {
    if !mojibake_markers(input) {
        return Ok(input.to_string());
    }
    let mut bytes = Vec::with_capacity(input.len());
    for c in input.chars() {
        match char_to_latin_byte(c) {
            Some(b) => bytes.push(b),
            None => {
                return Err(format!(
                    "The text looks like mojibake (UTF-8 read as Latin-1/Windows-1252), but the character '{c}' can't be mapped back to a single byte, so a full repair isn't possible."
                ));
            }
        }
    }
    match String::from_utf8(bytes) {
        Ok(repaired) => Ok(repaired),
        Err(_) => Err("The text looks like mojibake (UTF-8 read as Latin-1/Windows-1252), but repairing it didn't produce valid UTF-8 — it may be a different mix of encodings.".into()),
    }
}

// ---------------------------------------------------------------------------
// 21. Invisible Character Revealer
// ---------------------------------------------------------------------------

const INVISIBLES: &[(char, &str)] = &[
    ('\u{200B}', "ZERO WIDTH SPACE"),
    ('\u{200C}', "ZERO WIDTH NON-JOINER"),
    ('\u{200D}', "ZERO WIDTH JOINER"),
    ('\u{2060}', "WORD JOINER"),
    ('\u{FEFF}', "ZERO WIDTH NO-BREAK SPACE / BYTE ORDER MARK"),
    ('\u{00AD}', "SOFT HYPHEN"),
    ('\u{180E}', "MONGOLIAN VOWEL SEPARATOR"),
    ('\u{202A}', "LEFT-TO-RIGHT EMBEDDING"),
    ('\u{202B}', "RIGHT-TO-LEFT EMBEDDING"),
    ('\u{202C}', "POP DIRECTIONAL FORMATTING"),
    ('\u{202D}', "LEFT-TO-RIGHT OVERRIDE"),
    ('\u{202E}', "RIGHT-TO-LEFT OVERRIDE"),
    ('\u{2066}', "LEFT-TO-RIGHT ISOLATE"),
    ('\u{2067}', "RIGHT-TO-LEFT ISOLATE"),
    ('\u{2068}', "FIRST STRONG ISOLATE"),
    ('\u{2069}', "POP DIRECTIONAL ISOLATE"),
];

fn invisible_name(c: char) -> Option<&'static str> {
    INVISIBLES.iter().find(|(ch, _)| *ch == c).map(|(_, n)| *n)
}

fn invisible_reveal(input: &str, _param: &str) -> Result<String, String> {
    let mut count = 0usize;
    let mut out = String::new();
    for c in input.chars() {
        if let Some(name) = invisible_name(c) {
            count += 1;
            out.push_str(&format!("<U+{:04X} {}>", c as u32, name));
        } else {
            out.push(c);
        }
    }
    if count == 0 {
        return Ok("No invisible characters found.".into());
    }
    Ok(format!("{count} invisible character(s) found:\n{out}"))
}

// ---------------------------------------------------------------------------
// 22. Line Endings (inspect / convert)
// ---------------------------------------------------------------------------

fn line_endings(input: &str, param: &str) -> Result<String, String> {
    let mode = param.trim().to_uppercase();
    let b = input.as_bytes();
    let (mut crlf, mut lf, mut cr) = (0usize, 0usize, 0usize);
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\r' {
            if b.get(i + 1) == Some(&b'\n') {
                crlf += 1;
                i += 2;
            } else {
                cr += 1;
                i += 1;
            }
        } else if b[i] == b'\n' {
            lf += 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    match mode.as_str() {
        "" => {
            let dominant = if crlf == 0 && lf == 0 && cr == 0 {
                "none — the text has no line breaks"
            } else if crlf >= lf && crlf >= cr {
                "CRLF"
            } else if lf >= cr {
                "LF"
            } else {
                "CR"
            };
            Ok(format!(
                "Line endings found:\nCRLF (Windows): {crlf}\nLF only (Unix/macOS): {lf}\nCR only (very old Mac): {cr}\nDominant ending: {dominant}\n\nTip: set the Parameter to LF, CRLF or CR to convert everything to that ending."
            ))
        }
        "LF" | "CRLF" | "CR" => {
            let target = match mode.as_str() {
                "LF" => "\n",
                "CRLF" => "\r\n",
                _ => "\r",
            };
            let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
            Ok(normalized.replace('\n', target))
        }
        _ => Err(format!(
            "The Parameter must be LF, CRLF or CR (or left empty to just inspect). Got '{param}'."
        )),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_build_signs_verifies_and_errors() {
        let out = jwt_build("{\"sub\":\"123\",\"name\":\"Ada\"}", "secret").unwrap();
        let token = out.lines().next().unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        // The signature must be a real HMAC-SHA256 of header.payload.
        let sig = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
        let expect = hmac_sha256(b"secret", format!("{}.{}", parts[0], parts[1]).as_bytes());
        assert_eq!(sig, expect);
        let payload = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
        assert!(String::from_utf8(payload).unwrap().contains("Ada"));
        assert!(out.contains("Payload:"));
        // Extra header fields are merged in.
        let with_kid = jwt_build("{}", "secret|||{\"kid\":\"k1\"}").unwrap();
        assert!(with_kid.contains("\"kid\""));
        // Errors.
        assert!(jwt_build("[1,2]", "secret").is_err());
        assert!(jwt_build("{}", "").is_err());
        assert!(jwt_build("{}", "secret|||not json").is_err());
        assert!(jwt_build("not json", "secret").is_err());
    }

    #[test]
    fn number_identify_readings() {
        let unix = number_identify("1700000000", "").unwrap();
        assert!(unix.contains("As Unix time (seconds): 2023-11-14 22:13:20 UTC"));
        let ft = number_identify("133170048000000000", "").unwrap();
        assert!(ft.contains("Windows FILETIME"));
        assert!(ft.contains("2023-01-01"));
        let hi = number_identify("18537", "").unwrap();
        assert!(hi.contains("0x4869"));
        assert!(hi.contains("As ASCII text: \"Hi\""));
        let excel = number_identify("44927", "").unwrap();
        assert!(excel.contains("As an Excel serial date: 2023-01-01"));
        let hexin = number_identify("0xFF", "").unwrap();
        assert!(hexin.contains("Binary: 0b11111111"));
        assert!(number_identify("hello", "").is_err());
        assert!(number_identify("   ", "").is_err());
    }

    #[test]
    fn json_diff_reports_and_errors() {
        let a = "{\"a\":1,\"b\":{\"c\":2},\"list\":[1,2]}";
        let b = "{\"a\":1,\"b\":{\"c\":3},\"list\":[1,2,4],\"d\":true}";
        let out = json_diff(a, b).unwrap();
        assert!(out.contains("$.b.c: 2 → 3"));
        assert!(out.contains("$.list[2]: only in second"));
        assert!(out.contains("$.d: only in second"));
        assert!(json_diff(a, a).unwrap().contains("structurally identical"));
        assert!(json_diff("nope", b).unwrap_err().contains("Input"));
        assert!(json_diff(a, "nope").unwrap_err().contains("Parameter"));
        assert!(json_diff(a, "").is_err());
    }

    #[test]
    fn ini_converters_round_trip_shapes() {
        let json = ini_to_json("[server]\nhost = example.com\nport = 8080\n", "").unwrap();
        assert!(json.contains("\"server\""));
        assert!(json.contains("\"host\": \"example.com\""));
        assert!(json.contains("\"port\": \"8080\""));
        let dotted = ini_to_json("a.b = 1\n", "").unwrap();
        assert!(dotted.contains("\"b\": \"1\""));
        let ini = json_to_ini(
            "{\"name\":\"app\",\"db\":{\"host\":\"x\",\"port\":5432}}",
            "",
        )
        .unwrap();
        assert!(ini.contains("name = app"));
        assert!(ini.contains("[db]"));
        assert!(ini.contains("host = x"));
        assert!(ini.contains("port = 5432"));
        assert!(json_to_ini("[1,2]", "").is_err());
        assert!(json_to_ini("{\"a\":{\"b\":{\"c\":1}}}", "")
            .unwrap()
            .contains("[a.b]"));
    }

    #[test]
    fn env_converters() {
        let json = env_to_json("export A=1\nB=\"hello world\"\n# a note\nC='x'\n", "").unwrap();
        assert!(json.contains("\"A\": \"1\""));
        assert!(json.contains("\"B\": \"hello world\""));
        assert!(json.contains("\"C\": \"x\""));
        let env = json_to_env("{\"A\":\"hello world\",\"B\":2,\"C\":true}", "").unwrap();
        assert!(env.contains("A=\"hello world\""));
        assert!(env.contains("B=2"));
        assert!(env.contains("C=true"));
        assert!(json_to_env("{\"A\":{\"x\":1}}", "").is_err());
        assert!(env_to_json("nothing here", "").is_err());
    }

    #[test]
    fn env_diff_and_merge() {
        let diff = env_diff("X=1\nY=2\n", "X=1\nY=3\nZ=4").unwrap();
        assert!(diff.contains("Only in the first (0):"));
        assert!(diff.contains("Only in the second (1):"));
        assert!(diff.contains("Y: 2 → 3"));
        let masked = env_diff("X=1\nY=2\n", "X=1\nY=3|||mask").unwrap();
        assert!(masked.contains("Y: (differs)"));
        assert!(!masked.contains("2 → 3"));
        let merged = env_merge("B=1\nA=1\n", "B=2").unwrap();
        assert_eq!(merged, "A=1\nB=2");
        assert!(env_diff("X=1", "").is_err());
    }

    #[test]
    fn mock_data_fills_and_errors() {
        let out = mock_data(
            "{\"name\":\"{{fullName}}\",\"id\":\"{{uuid}}\",\"role\":\"{{choice:admin|user}}\",\"age\":30}",
            "3",
        )
        .unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(!out.contains("{{"));
        assert_eq!(arr[0]["age"], 30);
        assert!(arr[0]["name"].as_str().unwrap().contains(' '));
        let role = arr[0]["role"].as_str().unwrap();
        assert!(role == "admin" || role == "user");
        // Default count is 1; unknown placeholders error out.
        let one = mock_data("{\"w\":\"{{word}}\"}", "").unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&one)
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(mock_data("{\"x\":\"{{nope}}\"}", "").is_err());
        assert!(mock_data("{\"n\":\"{{int:5:2}}\"}", "").is_err());
        assert!(mock_data("\"just a string\"", "").is_err());
    }

    #[test]
    fn sql_in_list_and_expand() {
        assert_eq!(
            sql_in_list("a\nb\nO'Brien\n", "").unwrap(),
            "('a', 'b', 'O''Brien')"
        );
        assert_eq!(sql_in_list("1\n2.5\n", "numbers").unwrap(), "(1, 2.5)");
        assert!(sql_in_list("  \n", "").is_err());
        assert!(sql_in_list("a\n", "numbers").is_err());

        let expanded = sql_expand("SELECT * FROM t WHERE a = ? AND b = ?", "42|||O'Brien").unwrap();
        assert_eq!(expanded, "SELECT * FROM t WHERE a = 42 AND b = 'O''Brien'");
        assert!(sql_expand("SELECT ?", "1|||2").is_err());
        assert!(sql_expand("SELECT ?", "").is_err());
        // A ? inside a quoted string isn't a placeholder.
        assert_eq!(sql_expand("SELECT '?'", "").unwrap(), "SELECT '?'");
        assert_eq!(sql_expand("SELECT ?", "null").unwrap(), "SELECT NULL");
    }

    #[test]
    fn sql_create_table_infers_types() {
        let create = sql_create_table("id,name,score\n1,ada,9.5\n2,bob,8\n", "people").unwrap();
        assert!(create.contains("CREATE TABLE \"people\""));
        assert!(create.contains("\"id\" INTEGER"));
        assert!(create.contains("\"name\" TEXT"));
        assert!(create.contains("\"score\" REAL"));
        let from_json =
            sql_create_table("[{\"a\": 1, \"ok\": true}, {\"a\": 2, \"ok\": false}]", "").unwrap();
        assert!(from_json.contains("\"a\" INTEGER"));
        assert!(from_json.contains("\"ok\" BOOLEAN"));
        assert!(from_json.contains("\"my_table\""));
        assert!(sql_create_table("", "").is_err());
    }

    #[test]
    fn gitignore_builds_and_errors() {
        let gi = gitignore_gen("", "rust, python").unwrap();
        assert!(gi.contains("# ===== rust ====="));
        assert!(gi.contains("target/"));
        assert!(gi.contains("# ===== python ====="));
        assert!(gi.contains("__pycache__/"));
        let err = gitignore_gen("", "rust, cobol").unwrap_err();
        assert!(err.contains("cobol"));
        assert!(err.contains("Valid names"));
        assert!(gitignore_gen("", "").is_err());
    }

    #[test]
    fn commit_check_verdicts() {
        let good = commit_check("feat(parser): add a thing", "").unwrap();
        assert!(good.contains("✓"));
        assert!(good.contains("Overall: looks good"));
        let bad = commit_check("fixed stuff.", "").unwrap();
        assert!(bad.contains("✗"));
        assert!(bad.contains("Overall:"));
        let unknown = commit_check("wip: halfway there", "").unwrap();
        assert!(unknown.contains("isn't a conventional type"));
        let too_long = commit_check(
            "fix: this subject line is deliberately made far too long so that it exceeds the limit",
            "",
        )
        .unwrap();
        assert!(too_long.contains("72"));
        assert!(too_long.contains("✗"));
        assert!(commit_check("   ", "").is_err());
    }

    #[test]
    fn changelog_groups_commits() {
        let out = changelog_from_commits(
            "a1b2c3d feat(api): add login\nfix: repair crash\nrandom tidy up\n",
            "",
        )
        .unwrap();
        assert!(out.contains("## Features"));
        assert!(out.contains("- add login (api)"));
        assert!(out.contains("## Fixes"));
        assert!(out.contains("- repair crash"));
        assert!(out.contains("## Other Changes"));
        assert!(out.contains("- random tidy up"));
        assert!(changelog_from_commits("  ", "").is_err());
    }

    #[test]
    fn regex_explain_elements() {
        let out = regex_explain("\\d{2,4}[a-z]+$", "").unwrap();
        assert!(out.contains("a digit (0–9)"));
        assert!(out.contains("between 2 and 4 times"));
        assert!(out.contains("a character class: any of 'a' to 'z'"));
        assert!(out.contains("one or more times"));
        assert!(out.contains("end of the text"));
        let grouped = regex_explain("(ab|cd)", "").unwrap();
        assert!(grouped.contains("capturing group 1"));
        assert!(grouped.contains("alternation"));
        let named = regex_explain("(?<year>\\d{4})", "").unwrap();
        assert!(named.contains("named capturing group"));
        assert!(named.contains("year"));
        assert!(regex_explain("", "").is_err());
    }

    #[test]
    fn log_analyze_summary() {
        let sample = "2024-01-01 10:00:00 INFO starting\n2024-01-01 10:00:01 ERROR failed to connect to 192.168.0.1\n2024-01-01 10:00:02 ERROR failed to connect to 10.0.0.9\n2024-01-01 10:00:03 WARN slow request 512 ms\n";
        let out = log_analyze(sample, "").unwrap();
        assert!(out.contains("Total lines: 4"));
        assert!(out.contains("ERROR (including FATAL and ERR): 2"));
        assert!(out.contains("WARN: 1"));
        assert!(out.contains("Busiest level: ERROR (2 lines)"));
        assert!(out.contains("First timestamp: 2024-01-01 10:00:00"));
        assert!(out.contains("Last timestamp: 2024-01-01 10:00:03"));
        assert!(out.contains("2 × "));
        assert!(out.contains("ERROR failed to connect to #.#.#.#"));
        assert!(log_analyze("   ", "").is_err());
    }

    #[test]
    fn stack_clean_folds_and_shortens() {
        let trace = "Traceback (most recent call last):\n  File \"/home/user/projects/app/src/main.py\", line 10, in <module>\n  File \"/home/user/projects/app/src/main.py\", line 10, in <module>\n    boom()\n";
        let out = stack_clean(trace, "").unwrap();
        assert!(out.contains("…/src/main.py"));
        assert!(out.contains("×2"));
        assert!(out.contains("2 frames"));
        assert!(out.contains("lines after cleaning"));
        assert!(stack_clean("  ", "").is_err());
    }

    #[test]
    fn mojibake_repairs() {
        assert_eq!(mojibake_fix("cafÃ©", "").unwrap(), "café");
        let quotes = mojibake_fix("â€œhello", "").unwrap();
        assert!(quotes.contains("“hello"));
        let clean = mojibake_fix("plain text", "").unwrap();
        assert_eq!(clean, "plain text");
        // A marker followed by a dangling lead byte can't repair.
        assert!(mojibake_fix("cafÃ© Ã", "").is_err());
    }

    #[test]
    fn invisible_reveal_tags() {
        let out = invisible_reveal("a\u{200B}b", "").unwrap();
        assert!(out.contains("1 invisible character(s) found:"));
        assert!(out.contains("<U+200B ZERO WIDTH SPACE>"));
        assert_eq!(
            invisible_reveal("clean", "").unwrap(),
            "No invisible characters found."
        );
    }

    #[test]
    fn line_endings_report_and_convert() {
        let report = line_endings("a\r\nb\nc\r\n", "").unwrap();
        assert!(report.contains("CRLF (Windows): 2"));
        assert!(report.contains("LF only (Unix/macOS): 1"));
        assert!(report.contains("Dominant ending: CRLF"));
        assert_eq!(line_endings("a\r\nb\r\n", "LF").unwrap(), "a\nb\n");
        assert_eq!(line_endings("a\nb\n", "CRLF").unwrap(), "a\r\nb\r\n");
        assert!(line_endings("x", "bogus").is_err());
    }
}
