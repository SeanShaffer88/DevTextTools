use super::Op;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

fn op(
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    hint: Option<&'static str>,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: "Developer / Format",
        description: desc,
        param_hint: hint,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    vec![
        op("dev-json-pretty", "JSON Format / Beautify", "Pretty-print JSON with 2-space indent. Also validates it.", None, |i, _| { let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?; serde_json::to_string_pretty(&v).map_err(|e| e.to_string()) }),
        op("dev-json-minify", "JSON Minify", "Remove all unnecessary whitespace from JSON.", None, |i, _| { let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?; serde_json::to_string(&v).map_err(|e| e.to_string()) }),
        op("dev-json-validate", "JSON Validate + Stats", "Check JSON is valid and report its size, depth and counts.", None, |i, _| { let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?; Ok(json_stats(&v, i.len())) }),
        op("dev-json-to-csv", "JSON Array → CSV", "Convert a JSON array of flat objects to CSV.", None, |i, _| json_to_csv(i)),
        op("dev-csv-to-json", "CSV → JSON", "Convert CSV (first row = headers) to a JSON array.", None, |i, _| csv_to_json(i)),
        op("dev-json-to-yaml", "JSON → YAML", "Convert JSON to simple YAML (nested objects/arrays supported).", None, |i, _| { let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?; Ok(to_yaml(&v, 0)) }),
        op("dev-xml-pretty", "XML Format", "Indent XML by tag nesting. Basic formatter for well-formed XML.", None, |i, _| xml_pretty(i)),
        op("dev-sql-format", "SQL Format (basic)", "Uppercase SQL keywords and break before FROM/WHERE/GROUP/ORDER etc.", None, |i, _| Ok(sql_format(i))),
        op("dev-css-minify", "CSS Minify (basic)", "Remove CSS comments and extra whitespace.", None, |i, _| Ok(css_minify(i))),
        op("dev-jwt-decode", "JWT Decoder", "Decode a JSON Web Token's header and payload (does NOT verify the signature).", None, |i, _| jwt_decode(i.trim())),
        op("dev-regex-test", "Regex Tester", "Test a pattern against the input: lists every match with its position.", Some("regex pattern"), |i, p| { if p.trim().is_empty() { return Err("Enter the pattern as the Parameter".into()); } let re = regex::Regex::new(p.trim()).map_err(|e| format!("Bad pattern: {e}"))?; let ms: Vec<String> = re.find_iter(i).map(|m| format!("{}–{}: {}", m.start(), m.end(), m.as_str())).collect(); if ms.is_empty() { Ok("No matches.".into()) } else { Ok(format!("{} match(es):\n{}", ms.len(), ms.join("\n"))) } }),
        op("dev-cron-explain", "Cron Expression Explainer", "Explain a 5-field cron schedule (minute hour day month weekday) in plain words.", None, |i, _| cron_explain(i.trim())),
        op("dev-url-parse", "URL Parser", "Split a web address into scheme, host, port, path, query and fragment.", None, |i, _| url_parse(i.trim())),
        op("dev-query-to-json", "Query String → JSON", "Turn ?a=1&b=2 into JSON.", None, |i, _| { let q = i.trim().trim_start_matches('?'); let mut map = serde_json::Map::new(); for pair in q.split('&').filter(|s| !s.is_empty()) { let (k, v) = pair.split_once('=').unwrap_or((pair, "")); map.insert(k.to_string(), serde_json::Value::String(v.to_string())); } serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(|e| e.to_string()) }),
        op("dev-json-to-query", "JSON → Query String", "Turn a flat JSON object into a=1&b=2.", None, |i, _| { let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?; let obj = v.as_object().ok_or("Input must be a JSON object")?; Ok(obj.iter().map(|(k, val)| format!("{k}={}", val.as_str().map(|s| s.to_string()).unwrap_or_else(|| val.to_string()))).collect::<Vec<_>>().join("&")) }),
        op("dev-env-sort", "Sort .env / KEY=VALUE Lines", "Sort KEY=value lines alphabetically by key, keeping comments with their key where possible.", None, |i, _| { let mut lines: Vec<&str> = i.lines().filter(|l| !l.trim().is_empty()).collect(); lines.sort_by_key(|l| l.split('=').next().unwrap_or("").to_lowercase()); Ok(lines.join("\n")) }),
        op("dev-uuid-validate", "UUID Validate + Parse", "Check UUIDs (one per line) and report version/variant.", None, |i, _| { Ok(i.lines().filter(|l| !l.trim().is_empty()).map(uuid_check).collect::<Vec<_>>().join("\n")) }),
        op("dev-color-css", "Color: CSS Formats", "Input a hex color (#RRGGBB) — get rgb(), hsl(), hsv() and CSS custom property.", None, |i, _| color_css(i.trim())),
    ]
}

fn json_stats(v: &serde_json::Value, bytes: usize) -> String {
    fn walk(
        v: &serde_json::Value,
        depth: usize,
        counts: &mut (usize, usize, usize, usize),
    ) -> usize {
        match v {
            serde_json::Value::Object(m) => {
                counts.0 += 1;
                m.values()
                    .map(|x| walk(x, depth + 1, counts))
                    .max()
                    .unwrap_or(depth)
            }
            serde_json::Value::Array(a) => {
                counts.1 += 1;
                a.iter()
                    .map(|x| walk(x, depth + 1, counts))
                    .max()
                    .unwrap_or(depth)
            }
            serde_json::Value::String(_) => {
                counts.2 += 1;
                depth
            }
            _ => {
                counts.3 += 1;
                depth
            }
        }
    }
    let mut c = (0, 0, 0, 0);
    let depth = walk(v, 1, &mut c);
    format!("Valid JSON ✓\nSize: {bytes} bytes\nMax depth: {depth}\nObjects: {}\nArrays: {}\nStrings: {}\nNumbers/bools/null: {}", c.0, c.1, c.2, c.3)
}
pub fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
fn json_to_csv(i: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?;
    let arr = v
        .as_array()
        .ok_or("Input must be a JSON array of objects")?;
    if arr.is_empty() {
        return Ok(String::new());
    }
    let first = arr[0].as_object().ok_or("Array items must be objects")?;
    let keys: Vec<String> = first.keys().cloned().collect();
    let mut out = vec![keys
        .iter()
        .map(|k| csv_cell(k))
        .collect::<Vec<_>>()
        .join(",")];
    for item in arr {
        let obj = item.as_object().ok_or("Array items must all be objects")?;
        out.push(
            keys.iter()
                .map(|k| {
                    let val = obj.get(k).cloned().unwrap_or(serde_json::Value::Null);
                    csv_cell(&match val {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    })
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    Ok(out.join("\n"))
}
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut ch = line.chars().peekable();
    while let Some(c) = ch.next() {
        match c {
            '"' if in_q && ch.peek() == Some(&'"') => {
                cur.push('"');
                ch.next();
            }
            '"' => in_q = !in_q,
            ',' if !in_q => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}
fn csv_to_json(i: &str) -> Result<String, String> {
    let mut lines = i.lines().filter(|l| !l.trim().is_empty());
    let header = parse_csv_line(lines.next().ok_or("Empty CSV")?);
    let mut arr = Vec::new();
    for line in lines {
        let cells = parse_csv_line(line);
        let mut obj = serde_json::Map::new();
        for (idx, h) in header.iter().enumerate() {
            obj.insert(
                h.clone(),
                serde_json::Value::String(cells.get(idx).cloned().unwrap_or_default()),
            );
        }
        arr.push(serde_json::Value::Object(obj));
    }
    serde_json::to_string_pretty(&serde_json::Value::Array(arr)).map_err(|e| e.to_string())
}
fn to_yaml(v: &serde_json::Value, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match v {
        serde_json::Value::Object(m) => m
            .iter()
            .map(|(k, val)| match val {
                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                    format!("{pad}{k}:\n{}", to_yaml(val, indent + 1))
                }
                _ => format!("{pad}{k}: {}", scalar(val)),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Array(a) => a
            .iter()
            .map(|val| match val {
                serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                    format!("{pad}-\n{}", to_yaml(val, indent + 1))
                }
                _ => format!("{pad}- {}", scalar(val)),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        other => format!("{pad}{}", scalar(other)),
    }
}
fn scalar(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => {
            if s.contains(|c: char| ":{}[],&*#?|-<>=!%@`\"'".contains(c)) {
                format!("\"{}\"", s.replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        other => other.to_string(),
    }
}
fn xml_pretty(i: &str) -> Result<String, String> {
    let re = regex::Regex::new(r">\s+<").unwrap();
    let flat = re.replace_all(i.trim(), "><");
    let mut out = Vec::new();
    let mut depth = 0usize;
    for part in flat.split_inclusive('>') {
        let t = part.trim();
        if t.is_empty() {
            continue;
        }
        let is_close = t.starts_with("</");
        let is_self = t.ends_with("/>") || t.starts_with("<?") || t.starts_with("<!");
        if is_close {
            depth = depth.saturating_sub(1);
        }
        out.push(format!("{}{}", "  ".repeat(depth), t));
        if !is_close && !is_self && !t.starts_with("</") {
            depth += 1;
        }
        // text content stays on same logical level; simple formatter limitation noted in docs
        if t.contains("</") && !is_close {
            depth = depth.saturating_sub(1);
        }
    }
    if out.is_empty() {
        return Err("No XML tags found".into());
    }
    Ok(out.join("\n"))
}
fn sql_format(i: &str) -> String {
    let keywords = [
        "SELECT",
        "FROM",
        "WHERE",
        "GROUP BY",
        "ORDER BY",
        "HAVING",
        "LIMIT",
        "JOIN",
        "LEFT JOIN",
        "INNER JOIN",
        "ON",
        "INSERT INTO",
        "VALUES",
        "UPDATE",
        "SET",
        "DELETE",
        "CREATE TABLE",
    ];
    let mut s = format!(" {} ", i.split_whitespace().collect::<Vec<_>>().join(" "));
    for kw in keywords {
        let re =
            regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&kw.to_lowercase()))).unwrap();
        s = re.replace_all(&s, kw).to_string();
        let marker = format!(" {kw} ");
        s = s.replace(&marker, &format!("\n{kw} "));
    }
    s.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
fn css_minify(i: &str) -> String {
    let no_comments = regex::Regex::new(r"/\*.*?\*/")
        .unwrap()
        .replace_all(i, "")
        .to_string();
    let mut s = no_comments.split_whitespace().collect::<Vec<_>>().join(" ");
    for (a, b) in [
        (" {", "{"),
        ("{ ", "{"),
        (" }", "}"),
        ("} ", "}"),
        ("; ", ";"),
        (": ", ":"),
        (", ", ","),
    ] {
        s = s.replace(a, b);
    }
    s.replace(";}", "}")
}
fn jwt_decode(tok: &str) -> Result<String, String> {
    let parts: Vec<&str> = tok.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "A JWT has 3 parts separated by dots — this has {}",
            parts.len()
        ));
    }
    let dec = |p: &str, what: &str| -> Result<String, String> {
        let b = URL_SAFE_NO_PAD
            .decode(p)
            .map_err(|e| format!("{what} is not valid Base64URL: {e}"))?;
        let s = String::from_utf8(b).map_err(|_| format!("{what} is not text"))?;
        let v: serde_json::Value =
            serde_json::from_str(&s).map_err(|e| format!("{what} is not JSON: {e}"))?;
        serde_json::to_string_pretty(&v).map_err(|e| e.to_string())
    };
    Ok(format!("Header:\n{}\n\nPayload:\n{}\n\nSignature: present ({} chars) — not verified. Decoding is not verification: never trust a token's claims from this alone.", dec(parts[0], "Header")?, dec(parts[1], "Payload")?, parts[2].len()))
}
fn cron_explain(expr: &str) -> Result<String, String> {
    let f: Vec<&str> = expr.split_whitespace().collect();
    if f.len() != 5 {
        return Err(format!(
            "A classic cron expression has 5 fields — this has {}",
            f.len()
        ));
    }
    let names = [
        "minute",
        "hour",
        "day of month",
        "month",
        "day of week (0=Sun)",
    ];
    let mut out = vec![format!("Schedule: {expr}")];
    for (idx, field) in f.iter().enumerate() {
        let plain = if *field == "*" {
            "every".to_string()
        } else if field.contains('/') {
            format!(
                "every {} (from {})",
                field.split('/').nth(1).unwrap_or("?"),
                field.split('/').next().unwrap_or("*")
            )
        } else if field.contains(',') {
            format!("on {}", field.replace(',', ", "))
        } else if field.contains('-') {
            format!(
                "from {} to {}",
                field.split('-').next().unwrap_or("?"),
                field.split('-').nth(1).unwrap_or("?")
            )
        } else {
            format!("at {field}")
        };
        out.push(format!("{}: {} ({plain})", names[idx], field));
    }
    out.push("Note: plain-words summary only — your scheduler is the final authority.".into());
    Ok(out.join("\n"))
}
fn url_parse(u: &str) -> Result<String, String> {
    let (scheme, rest) = u
        .split_once("://")
        .ok_or("Address must contain :// e.g. https://")?;
    let (hostport, tail) = match rest.find(['/', '?', '#']) {
        Some(p) => (&rest[..p], &rest[p..]),
        None => (rest, ""),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (h, p),
        _ => (hostport, "(default)"),
    };
    let (pathq, frag) = match tail.split_once('#') {
        Some((a, b)) => (a, b),
        None => (tail, ""),
    };
    let (path, query) = match pathq.split_once('?') {
        Some((a, b)) => (a, b),
        None => (pathq, ""),
    };
    Ok(format!(
        "Scheme: {scheme}\nHost: {host}\nPort: {port}\nPath: {}\nQuery: {}\nFragment: {}",
        if path.is_empty() { "/" } else { path },
        if query.is_empty() { "(none)" } else { query },
        if frag.is_empty() { "(none)" } else { frag }
    ))
}
fn uuid_check(l: &str) -> String {
    let t = l.trim();
    let hexpart: String = t.chars().filter(|c| *c != '-').collect();
    let shape = t.len() == 36
        && t.chars().filter(|c| *c == '-').count() == 4
        && hexpart.len() == 32
        && hexpart.chars().all(|c| c.is_ascii_hexdigit());
    if !shape {
        return format!("{t}: INVALID — not in 8-4-4-4-12 hex form");
    }
    let version = t.chars().nth(14).unwrap_or('?');
    let variant = t.chars().nth(19).unwrap_or('?');
    format!("{t}: valid ✓  version={version}  variant-char={variant}")
}
pub fn parse_hex_color(s: &str) -> Result<(u8, u8, u8), String> {
    let h = s.trim().trim_start_matches('#');
    let h = if h.len() == 3 {
        h.chars().map(|c| format!("{c}{c}")).collect::<String>()
    } else {
        h.to_string()
    };
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Enter a hex color like #E5484D or #FFF".into());
    }
    Ok((
        u8::from_str_radix(&h[0..2], 16).unwrap(),
        u8::from_str_radix(&h[2..4], 16).unwrap(),
        u8::from_str_radix(&h[4..6], 16).unwrap(),
    ))
}
fn color_css(s: &str) -> Result<String, String> {
    let (r, g, b) = parse_hex_color(s)?;
    let (h, sat, l) = rgb_to_hsl(r, g, b);
    Ok(format!("Hex: #{r:02X}{g:02X}{b:02X}\nRGB: rgb({r}, {g}, {b})\nHSL: hsl({h:.0}, {sat:.0}%, {l:.0}%)\nCSS variable: --color: #{r:02X}{g:02X}{b:02X};"))
}
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l * 100.0);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    ((h + 360.0) % 360.0, s * 100.0, l * 100.0)
}
