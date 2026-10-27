use super::Op;

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

/// Map an ASCII char into a mathematical-alphanumeric block starting at
/// `cap`/`small`/`digit` code points. Non-ASCII-alphanumerics pass through.
fn block_map(c: char, cap: u32, small: u32, digit: Option<u32>) -> char {
    let cp = match c {
        'A'..='Z' => Some(cap + (c as u32 - 'A' as u32)),
        'a'..='z' => Some(small + (c as u32 - 'a' as u32)),
        '0'..='9' => digit.map(|d| d + (c as u32 - '0' as u32)),
        _ => None,
    };
    match cp {
        Some(n) => char::from_u32(n).unwrap_or(c),
        None => c,
    }
}

fn styled(i: &str, f: fn(char) -> char) -> Result<String, String> {
    Ok(i.chars().map(f).collect())
}

fn script_char(c: char) -> char {
    // Mathematical script has several letters living in the letterlike block.
    match c {
        'B' => '\u{212C}',
        'E' => '\u{2130}',
        'F' => '\u{2131}',
        'H' => '\u{210B}',
        'I' => '\u{2110}',
        'L' => '\u{2112}',
        'M' => '\u{2133}',
        'R' => '\u{211B}',
        'e' => '\u{212F}',
        'g' => '\u{210A}',
        'o' => '\u{2134}',
        _ => block_map(c, 0x1D49C, 0x1D4B6, None),
    }
}
fn fraktur_char(c: char) -> char {
    match c {
        'C' => '\u{212D}',
        'H' => '\u{210C}',
        'I' => '\u{2111}',
        'R' => '\u{211C}',
        'Z' => '\u{2128}',
        _ => block_map(c, 0x1D504, 0x1D51E, None),
    }
}
fn doublestruck_char(c: char) -> char {
    match c {
        'C' => '\u{2102}',
        'H' => '\u{210D}',
        'N' => '\u{2115}',
        'P' => '\u{2119}',
        'Q' => '\u{211A}',
        'R' => '\u{211D}',
        'Z' => '\u{2124}',
        _ => block_map(c, 0x1D538, 0x1D552, Some(0x1D7D8)),
    }
}
fn italic_char(c: char) -> char {
    if c == 'h' {
        return '\u{210E}'; // Planck constant sign is the italic h
    }
    block_map(c, 0x1D434, 0x1D44E, None)
}
fn circled_char(c: char) -> char {
    match c {
        'A'..='Z' => char::from_u32(0x24B6 + (c as u32 - 'A' as u32)).unwrap_or(c),
        'a'..='z' => char::from_u32(0x24D0 + (c as u32 - 'a' as u32)).unwrap_or(c),
        '0' => '\u{24EA}',
        '1'..='9' => char::from_u32(0x2460 + (c as u32 - '1' as u32)).unwrap_or(c),
        _ => c,
    }
}
fn smallcaps_char(c: char) -> char {
    match c.to_ascii_lowercase() {
        'a' => 'ᴀ',
        'b' => 'ʙ',
        'c' => 'ᴄ',
        'd' => 'ᴅ',
        'e' => 'ᴇ',
        'f' => 'ꜰ',
        'g' => 'ɢ',
        'h' => 'ʜ',
        'i' => 'ɪ',
        'j' => 'ᴊ',
        'k' => 'ᴋ',
        'l' => 'ʟ',
        'm' => 'ᴍ',
        'n' => 'ɴ',
        'o' => 'ᴏ',
        'p' => 'ᴘ',
        'q' => 'ǫ',
        'r' => 'ʀ',
        's' => 'ꜱ',
        't' => 'ᴛ',
        'u' => 'ᴜ',
        'v' => 'ᴠ',
        'w' => 'ᴡ',
        'x' => 'x',
        'y' => 'ʏ',
        'z' => 'ᴢ',
        _ => c,
    }
}
fn combining(i: &str, mark: char) -> Result<String, String> {
    let mut out = String::new();
    for c in i.chars() {
        out.push(c);
        if !c.is_whitespace() {
            out.push(mark);
        }
    }
    Ok(out)
}

fn vigenere(i: &str, p: &str, decode: bool) -> Result<String, String> {
    let key: Vec<i32> = p
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| (c.to_ascii_lowercase() as i32) - ('a' as i32))
        .collect();
    if key.is_empty() {
        return Err("Enter the key word as the Parameter (letters only)".into());
    }
    let mut ki = 0usize;
    let mut out = String::new();
    for c in i.chars() {
        if c.is_ascii_alphabetic() {
            let shift = if decode {
                -key[ki % key.len()]
            } else {
                key[ki % key.len()]
            };
            ki += 1;
            let base = if c.is_ascii_uppercase() { 'A' } else { 'a' } as i32;
            let n = ((c as i32 - base) + shift).rem_euclid(26);
            out.push(char::from_u32((base + n) as u32).unwrap_or(c));
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

pub fn ops() -> Vec<Op> {
    vec![
        op("fancy-bold", "Bold Text (Unicode)", "Fancy Text", "Convert letters/numbers to 𝗯𝗼𝗹𝗱 Unicode look-alikes. Note: this is styled characters, not real bold formatting — screen readers may read them oddly.", None, |i, _| styled(i, |c| block_map(c, 0x1D400, 0x1D41A, Some(0x1D7CE)))),
        op("fancy-italic", "Italic Text (Unicode)", "Fancy Text", "Convert letters to 𝑖𝑡𝑎𝑙𝑖𝑐 Unicode look-alikes.", None, |i, _| styled(i, italic_char)),
        op("fancy-bold-italic", "Bold Italic Text (Unicode)", "Fancy Text", "Convert letters to 𝒃𝒐𝒍𝒅 𝒊𝒕𝒂𝒍𝒊𝒄 Unicode look-alikes.", None, |i, _| styled(i, |c| block_map(c, 0x1D468, 0x1D482, None))),
        op("fancy-script", "Script / Cursive Text (Unicode)", "Fancy Text", "Convert letters to 𝓈𝒸𝓇𝒾𝓅𝓉 Unicode look-alikes.", None, |i, _| styled(i, script_char)),
        op("fancy-fraktur", "Fraktur / Old English Text (Unicode)", "Fancy Text", "Convert letters to 𝔣𝔯𝔞𝔨𝔱𝔲𝔯 Unicode look-alikes.", None, |i, _| styled(i, fraktur_char)),
        op("fancy-double-struck", "Double-Struck Text (Unicode)", "Fancy Text", "Convert letters/numbers to 𝕕𝕠𝕦𝕓𝕝𝕖-𝕤𝕥𝕣𝕦𝕔𝕜 look-alikes.", None, |i, _| styled(i, doublestruck_char)),
        op("fancy-monospace", "Monospace Text (Unicode)", "Fancy Text", "Convert letters/numbers to 𝚖𝚘𝚗𝚘𝚜𝚙𝚊𝚌𝚎 look-alikes.", None, |i, _| styled(i, |c| block_map(c, 0x1D670, 0x1D68A, Some(0x1D7F6)))),
        op("fancy-circled", "Circled Text (Unicode)", "Fancy Text", "Convert letters/numbers to ⓒⓘⓡⓒⓛⓔⓓ look-alikes.", None, |i, _| styled(i, circled_char)),
        op("fancy-small-caps", "Small Caps Text (Unicode)", "Fancy Text", "Convert text to ꜱᴍᴀʟʟ ᴄᴀᴘꜱ look-alikes.", None, |i, _| styled(i, smallcaps_char)),
        op("fancy-strikethrough", "Strikethrough Text (Unicode)", "Fancy Text", "Add a line through every character using a combining mark.", None, |i, _| combining(i, '\u{0336}')),
        op("fancy-underline", "Underlined Text (Unicode)", "Fancy Text", "Underline every character using a combining mark.", None, |i, _| combining(i, '\u{0332}')),
        op("tx-vigenere-encode", "Vigenère Cipher Encode", "Transform", "Encrypt with a key word: each letter is shifted by the repeating key. Parameter = key word.", Some("key word"), |i, p| vigenere(i, p, false)),
        op("tx-vigenere-decode", "Vigenère Cipher Decode", "Transform", "Decrypt Vigenère text with the same key word used to encrypt it.", Some("key word"), |i, p| vigenere(i, p, true)),
    ]
}
