use super::Op;
use crate::{split_param, Rng};
use sha1::Digest as _;

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
fn hexbytes(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn ops() -> Vec<Op> {
    vec![
        op("hash-md5", "MD5 Hash", "Hash / Crypto", "MD5 fingerprint of the text. For checksums only — MD5 is not safe for passwords/security.", None, |i, _| Ok(hexbytes(&md5::Md5::digest(i.as_bytes())))),
        op("hash-sha1", "SHA-1 Hash", "Hash / Crypto", "SHA-1 fingerprint. Legacy — prefer SHA-256 for new security uses.", None, |i, _| Ok(hexbytes(&sha1::Sha1::digest(i.as_bytes())))),
        op("hash-sha256", "SHA-256 Hash", "Hash / Crypto", "SHA-256 fingerprint of the text.", None, |i, _| Ok(hexbytes(&sha2::Sha256::digest(i.as_bytes())))),
        op("hash-sha384", "SHA-384 Hash", "Hash / Crypto", "SHA-384 fingerprint of the text.", None, |i, _| Ok(hexbytes(&sha2::Sha384::digest(i.as_bytes())))),
        op("hash-sha512", "SHA-512 Hash", "Hash / Crypto", "SHA-512 fingerprint of the text.", None, |i, _| Ok(hexbytes(&sha2::Sha512::digest(i.as_bytes())))),
        op("hash-all", "All Hashes at Once", "Hash / Crypto", "MD5, SHA-1, SHA-256, SHA-384, SHA-512 and CRC32 together.", None, |i, _| {
            Ok(format!("MD5:    {}\nSHA-1:  {}\nSHA-256:{}\nSHA-384:{}\nSHA-512:{}\nCRC32:  {:08x}", hexbytes(&md5::Md5::digest(i.as_bytes())), hexbytes(&sha1::Sha1::digest(i.as_bytes())), hexbytes(&sha2::Sha256::digest(i.as_bytes())), hexbytes(&sha2::Sha384::digest(i.as_bytes())), hexbytes(&sha2::Sha512::digest(i.as_bytes())), crc32(i.as_bytes())))
        }),
        op("hash-crc32", "CRC32 Checksum", "Hash / Crypto", "CRC32 checksum (the zip/file-integrity kind).", None, |i, _| Ok(format!("{:08x}", crc32(i.as_bytes())))),
        op("hash-hmac-sha256", "HMAC-SHA256", "Hash / Crypto", "Keyed HMAC-SHA256. Parameter is the secret key.", Some("secret key"), |i, p| { if p.is_empty() { return Err("Enter the secret key as the Parameter".into()); } Ok(hexbytes(&hmac_sha256(p.as_bytes(), i.as_bytes()))) }),
        op("gen-uuid", "UUID Generator", "Generators", "Generate random UUID v4 values. Input is ignored; parameter = how many (default 1).", Some("count (optional)"), |_, p| { let n = parse_count(p, 100)?; let mut rng = Rng::from_time(); Ok((0..n).map(|_| uuid_v4(&mut rng)).collect::<Vec<_>>().join("\n")) }),
        op("gen-guid-upper", "GUID Generator (Uppercase, Braces)", "Generators", "Windows-style {GUID} in uppercase.", Some("count (optional)"), |_, p| { let n = parse_count(p, 100)?; let mut rng = Rng::from_time(); Ok((0..n).map(|_| format!("{{{}}}", uuid_v4(&mut rng).to_uppercase())).collect::<Vec<_>>().join("\n")) }),
        op("gen-password", "Password Generator", "Generators", "Strong random passwords. Parameter: length|||count (defaults 20, 1). Uses a time-seeded generator — for high-stakes secrets prefer a password manager's generator.", Some("length|||count"), |_, p| { let v = split_param(p); let len = v.first().and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(20); let n = v.get(1).and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(1); if !(4..=512).contains(&len) { return Err("Length must be 4–512".into()); } if n > 100 { return Err("Max 100 passwords".into()); } let mut rng = Rng::from_time(); let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789!@#$%^&*()-_=+".chars().collect(); Ok((0..n).map(|_| (0..len).map(|_| chars[rng.below(chars.len() as u64) as usize]).collect::<String>()).collect::<Vec<_>>().join("\n")) }),
        op("gen-password-strength", "Password Strength Check", "Generators", "Estimate a password's strength: length, character variety, rough entropy. The password is never stored or sent anywhere.", None, |i, _| Ok(password_strength(i.trim()))),
        op("gen-random-string", "Random String", "Generators", "Random letters/numbers string. Parameter: length (default 32).", Some("length (optional)"), |_, p| { let len = p.trim().parse::<usize>().unwrap_or(32); if len > 10000 { return Err("Max length 10,000".into()); } let mut rng = Rng::from_time(); let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars().collect(); Ok((0..len).map(|_| chars[rng.below(chars.len() as u64) as usize]).collect()) }),
        op("gen-lorem", "Lorem Ipsum Generator", "Generators", "Filler paragraphs. Parameter: paragraph count (default 3).", Some("paragraphs (optional)"), |_, p| { let n = parse_count(p, 50)?; Ok((0..n).map(lorem_para).collect::<Vec<_>>().join("\n\n")) }),
        op("gen-random-number", "Random Number", "Generators", "Random integers. Parameter: min|||max|||count (defaults 1|||100|||10).", Some("min|||max|||count"), |_, p| { let v = split_param(p); let min: i64 = v.first().and_then(|s| s.trim().parse().ok()).unwrap_or(1); let max: i64 = v.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(100); let n: usize = v.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(10); if max <= min { return Err("Max must be bigger than min".into()); } if n > 10000 { return Err("Max 10,000 numbers".into()); } let mut rng = Rng::from_time(); let span = (max - min + 1) as u64; Ok((0..n).map(|_| (min + rng.below(span) as i64).to_string()).collect::<Vec<_>>().join("\n")) }),
        op("gen-dice", "Dice Roller", "Generators", "Roll dice. Input like 2d6 (two six-sided dice), parameter ignored.", None, |i, _| dice(i)),
        op("gen-coin", "Coin Flipper", "Generators", "Flip a coin N times. Parameter: count (default 1).", Some("count (optional)"), |_, p| { let n = parse_count(p, 10000)?; let mut rng = Rng::from_time(); let flips: Vec<&str> = (0..n).map(|_| if rng.below(2) == 0 { "Heads" } else { "Tails" }).collect(); Ok(format!("{}\nHeads: {}  Tails: {}", flips.join(", "), flips.iter().filter(|f| **f == "Heads").count(), flips.iter().filter(|f| **f == "Tails").count())) }),
        op("gen-pick-random-line", "Random Picker (pick from lines)", "Generators", "Pick random lines from the input list. Parameter: how many to pick (default 1).", Some("count (optional)"), |i, p| { let lines: Vec<&str> = i.lines().filter(|l| !l.trim().is_empty()).collect(); if lines.is_empty() { return Err("Enter the list, one item per line".into()); } let n = parse_count(p, lines.len().max(1))?; let mut rng = Rng::from_time(); Ok((0..n).map(|_| lines[rng.below(lines.len() as u64) as usize]).collect::<Vec<_>>().join("\n")) }),
        op("gen-random-color", "Random Color", "Generators", "Generate random hex colors. Parameter: count (default 5).", Some("count (optional)"), |_, p| { let n = parse_count(p, 100)?; let mut rng = Rng::from_time(); Ok((0..n).map(|_| { let v = rng.below(0x1000000); format!("#{v:06X}") }).collect::<Vec<_>>().join("\n")) }),
    ]
}

fn parse_count(p: &str, max: usize) -> Result<usize, String> {
    let n = p.trim().parse::<usize>().unwrap_or(1);
    if n == 0 || n > max {
        return Err(format!("Count must be 1–{max}"));
    }
    Ok(n)
}
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
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
pub fn uuid_v4(rng: &mut Rng) -> String {
    let mut b = [0u8; 16];
    for chunk in b.chunks_mut(8) {
        let v = rng.next_u64().to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15])
}
fn password_strength(pw: &str) -> String {
    let len = pw.chars().count();
    let mut pool = 0usize;
    if pw.chars().any(|c| c.is_lowercase()) {
        pool += 26;
    }
    if pw.chars().any(|c| c.is_uppercase()) {
        pool += 26;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        pool += 10;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) {
        pool += 33;
    }
    let entropy = if pool == 0 {
        0.0
    } else {
        len as f64 * (pool as f64).log2()
    };
    let verdict = if len == 0 {
        "empty"
    } else if entropy < 28.0 {
        "Very weak"
    } else if entropy < 45.0 {
        "Weak"
    } else if entropy < 65.0 {
        "Fair"
    } else if entropy < 90.0 {
        "Strong"
    } else {
        "Very strong"
    };
    format!("Length: {len} characters\nCharacter pool size: ~{pool}\nRough entropy: ~{entropy:.0} bits\nVerdict: {verdict}\n\nNote: a rough estimate only. Length matters most; use a password manager and a different password for every site.")
}
const LOREM: &[&str] = &[
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.",
    "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam.",
];
fn lorem_para(k: usize) -> String {
    LOREM[k % LOREM.len()].to_string()
}

fn dice(i: &str) -> Result<String, String> {
    let t = i.trim().to_lowercase();
    let (a, b) = t
        .split_once('d')
        .ok_or("Use the form 2d6 (count d sides)")?;
    let n: usize = if a.trim().is_empty() {
        1
    } else {
        a.trim().parse().map_err(|_| "Bad dice count".to_string())?
    };
    let sides: u64 = b
        .trim()
        .parse()
        .map_err(|_| "Use the form 2d6".to_string())?;
    if n == 0 || n > 1000 || sides < 2 {
        return Err("Count 1-1000, sides at least 2".into());
    }
    let mut rng = Rng::from_time();
    let rolls: Vec<u64> = (0..n).map(|_| rng.below(sides) + 1).collect();
    Ok(format!(
        "Rolls: {}\nTotal: {}",
        rolls
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(" + "),
        rolls.iter().sum::<u64>()
    ))
}
