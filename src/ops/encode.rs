use super::Op;
use base64::{engine::general_purpose::STANDARD, Engine};

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
        category: "Encode / Decode",
        description: desc,
        param_hint: hint,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    vec![
        op(
            "enc-base64-encode",
            "Base64 Encode",
            "Text → Base64.",
            None,
            |i, _| Ok(STANDARD.encode(i.as_bytes())),
        ),
        op(
            "enc-base64-decode",
            "Base64 Decode",
            "Base64 → text. Whitespace is ignored.",
            None,
            |i, _| {
                let clean: String = i.chars().filter(|c| !c.is_whitespace()).collect();
                let b = STANDARD
                    .decode(&clean)
                    .map_err(|e| format!("Invalid Base64: {e}"))?;
                String::from_utf8(b).map_err(|_| "Decoded bytes are not valid text (UTF-8)".into())
            },
        ),
        op(
            "enc-base64url-encode",
            "Base64 URL-Safe Encode",
            "Base64 with - and _ instead of + and /, no padding.",
            None,
            |i, _| Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(i.as_bytes())),
        ),
        op(
            "enc-base64url-decode",
            "Base64 URL-Safe Decode",
            "URL-safe Base64 → text.",
            None,
            |i, _| {
                let clean: String = i.chars().filter(|c| !c.is_whitespace()).collect();
                let b = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(&clean)
                    .map_err(|e| format!("Invalid Base64URL: {e}"))?;
                String::from_utf8(b).map_err(|_| "Decoded bytes are not valid text (UTF-8)".into())
            },
        ),
        op(
            "enc-base32-encode",
            "Base32 Encode",
            "Text → Base32 (A-Z2-7).",
            None,
            |i, _| Ok(base32_encode(i.as_bytes())),
        ),
        op(
            "enc-base32-decode",
            "Base32 Decode",
            "Base32 → text.",
            None,
            |i, _| {
                let b = base32_decode(i)?;
                String::from_utf8(b).map_err(|_| "Decoded bytes are not valid text".into())
            },
        ),
        op(
            "enc-hex-encode",
            "Hex Encode",
            "Text → hexadecimal bytes.",
            None,
            |i, _| Ok(i.bytes().map(|b| format!("{b:02x}")).collect()),
        ),
        op(
            "enc-hex-decode",
            "Hex Decode",
            "Hexadecimal → text. Spaces allowed.",
            None,
            |i, _| {
                let clean: String = i.chars().filter(|c| !c.is_whitespace()).collect();
                if clean.len() % 2 != 0 {
                    return Err("Hex text must have an even number of digits".into());
                }
                let mut b = Vec::new();
                let ch: Vec<char> = clean.chars().collect();
                for pair in ch.chunks(2) {
                    let s: String = pair.iter().collect();
                    b.push(u8::from_str_radix(&s, 16).map_err(|_| format!("'{s}' is not hex"))?);
                }
                String::from_utf8(b).map_err(|_| "Decoded bytes are not valid text".into())
            },
        ),
        op(
            "enc-binary-encode",
            "Binary Encode",
            "Text → 8-bit binary groups.",
            None,
            |i, _| {
                Ok(i.bytes()
                    .map(|b| format!("{b:08b}"))
                    .collect::<Vec<_>>()
                    .join(" "))
            },
        ),
        op(
            "enc-binary-decode",
            "Binary Decode",
            "8-bit binary groups → text.",
            None,
            |i, _| {
                let mut b = Vec::new();
                for g in i.split_whitespace() {
                    b.push(
                        u8::from_str_radix(g, 2)
                            .map_err(|_| format!("'{g}' is not an 8-bit binary group"))?,
                    );
                }
                String::from_utf8(b).map_err(|_| "Decoded bytes are not valid text".into())
            },
        ),
        op(
            "enc-url-encode",
            "URL Encode",
            "Percent-encode text for a web address.",
            None,
            |i, _| Ok(url_encode(i)),
        ),
        op(
            "enc-url-decode",
            "URL Decode",
            "Decode %XX percent-encoding (+ becomes space).",
            None,
            |i, _| url_decode(i),
        ),
        op(
            "enc-html-encode",
            "HTML Entity Encode",
            "Escape < > & \" ' as HTML entities.",
            None,
            |i, _| {
                Ok(i.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;"))
            },
        ),
        op(
            "enc-html-decode",
            "HTML Entity Decode",
            "Turn HTML entities (&amp; &#65; &#x41;) back into characters.",
            None,
            |i, _| html_decode(i),
        ),
        op(
            "enc-unicode-escape",
            "Unicode Escape",
            "Text → \\uXXXX escapes.",
            None,
            |i, _| {
                Ok(i.chars()
                    .map(|c| {
                        if c.is_ascii() && !c.is_control() {
                            c.to_string()
                        } else {
                            format!("\\u{:04x}", c as u32)
                        }
                    })
                    .collect())
            },
        ),
        op(
            "enc-unicode-unescape",
            "Unicode Unescape",
            " \\uXXXX escapes → text.",
            None,
            |i, _| unicode_unescape(i),
        ),
        op(
            "enc-codepoints",
            "Text → Code Points",
            "Show each character with its Unicode code point (U+XXXX) and name-ish details.",
            None,
            |i, _| {
                Ok(i.chars()
                    .map(|c| format!("{c}  U+{:04X}  decimal {}", c as u32, c as u32))
                    .collect::<Vec<_>>()
                    .join("\n"))
            },
        ),
        op(
            "enc-text-to-decimal",
            "Text → Decimal Bytes",
            "Text → space-separated byte numbers.",
            None,
            |i, _| {
                Ok(i.bytes()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(" "))
            },
        ),
        op(
            "enc-decimal-to-text",
            "Decimal Bytes → Text",
            "Space-separated byte numbers → text.",
            None,
            |i, _| {
                let mut b = Vec::new();
                for t in i
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter(|s| !s.is_empty())
                {
                    b.push(
                        t.parse::<u8>()
                            .map_err(|_| format!("'{t}' is not a byte (0-255)"))?,
                    );
                }
                String::from_utf8(b).map_err(|_| "Those bytes are not valid text".into())
            },
        ),
        op(
            "enc-quoted-printable-decode",
            "Quoted-Printable Decode",
            "Decode =XX quoted-printable (email) text.",
            None,
            |i, _| qp_decode(i),
        ),
        op(
            "enc-string-escape",
            "String Escape (C/JSON style)",
            "Escape quotes, backslashes and newlines for use inside code strings.",
            None,
            |i, _| {
                Ok(i.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t"))
            },
        ),
        op(
            "enc-string-unescape",
            "String Unescape",
            "Undo \\n \\t \\\" \\\\ escapes.",
            None,
            |i, _| Ok(unescape_str(i)),
        ),
    ]
}

const B32: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
fn base32_encode(data: &[u8]) -> String {
    let mut out = String::new();
    let mut buf: u32 = 0;
    let mut bits = 0;
    for &b in data {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(B32[((buf >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(B32[((buf << (5 - bits)) & 31) as usize] as char);
    }
    while out.len() % 8 != 0 {
        out.push('=');
    }
    out
}
fn base32_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0;
    for c in s.chars().filter(|c| !c.is_whitespace() && *c != '=') {
        let v = B32
            .iter()
            .position(|&x| x as char == c.to_ascii_uppercase())
            .ok_or(format!("'{c}' is not a Base32 character"))? as u32;
        buf = (buf << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}
fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
fn url_decode(s: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let ch: Vec<char> = s.chars().collect();
    let mut k = 0;
    while k < ch.len() {
        match ch[k] {
            '+' => {
                bytes.push(b' ');
                k += 1;
            }
            '%' => {
                if k + 2 >= ch.len() {
                    return Err("Incomplete %XX at end".into());
                }
                let h: String = ch[k + 1..k + 3].iter().collect();
                bytes.push(
                    u8::from_str_radix(&h, 16).map_err(|_| format!("'%{h}' is not valid hex"))?,
                );
                k += 3;
            }
            c => {
                let mut buf = [0u8; 4];
                bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                k += 1;
            }
        }
    }
    String::from_utf8(bytes).map_err(|_| "Result is not valid text".into())
}
fn html_decode(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos..];
        let end = after
            .find(';')
            .ok_or("An '&' has no closing ';' — not a valid entity")?;
        if end > 10 {
            return Err(format!(
                "'{}…' is not a valid HTML entity",
                &after[..after.len().min(12)]
            ));
        }
        let ent = &after[1..end];
        let decoded = match ent {
            "amp" => "&".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "quot" => "\"".to_string(),
            "apos" => "'".to_string(),
            "nbsp" => "\u{a0}".to_string(),
            _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                let n = u32::from_str_radix(&ent[2..], 16)
                    .map_err(|_| format!("Bad entity &{ent};"))?;
                char::from_u32(n)
                    .ok_or(format!("&{ent}; is not a real character"))?
                    .to_string()
            }
            _ if ent.starts_with('#') => {
                let n: u32 = ent[1..]
                    .parse()
                    .map_err(|_| format!("Bad entity &{ent};"))?;
                char::from_u32(n)
                    .ok_or(format!("&{ent}; is not a real character"))?
                    .to_string()
            }
            _ => return Err(format!("Unknown HTML entity &{ent};")),
        };
        out.push_str(&decoded);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
fn unicode_unescape(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let ch: Vec<char> = s.chars().collect();
    let mut k = 0;
    while k < ch.len() {
        if ch[k] == '\\' && k + 1 < ch.len() && ch[k + 1] == 'u' {
            if k + 5 >= ch.len() {
                return Err("Incomplete \\u escape at end".into());
            }
            let h: String = ch[k + 2..k + 6].iter().collect();
            let n =
                u32::from_str_radix(&h, 16).map_err(|_| format!("'\\u{h}' is not valid hex"))?;
            out.push(char::from_u32(n).ok_or(format!("\\u{h} is not a real character"))?);
            k += 6;
        } else {
            out.push(ch[k]);
            k += 1;
        }
    }
    Ok(out)
}
fn qp_decode(s: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let ch: Vec<char> = s.chars().collect();
    let mut k = 0;
    while k < ch.len() {
        if ch[k] == '=' {
            if k + 1 < ch.len() && ch[k + 1] == '\n' {
                k += 2;
                continue;
            }
            if k + 2 >= ch.len() {
                return Err("Incomplete =XX at end".into());
            }
            let h: String = ch[k + 1..k + 3].iter().collect();
            bytes.push(u8::from_str_radix(&h, 16).map_err(|_| format!("'={h}' is not valid"))?);
            k += 3;
        } else {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(ch[k].encode_utf8(&mut buf).as_bytes());
            k += 1;
        }
    }
    String::from_utf8(bytes).map_err(|_| "Result is not valid text".into())
}
fn unescape_str(s: &str) -> String {
    let mut out = String::new();
    let mut ch = s.chars();
    while let Some(c) = ch.next() {
        if c == '\\' {
            match ch.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
