use super::Op;
use crate::{numbers_from, split_param, Rng};

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
        // ---- Case ----
        op("case-upper", "Uppercase", "Case", "Convert all letters to UPPERCASE.", None, |i, _| Ok(i.to_uppercase())),
        op("case-lower", "Lowercase", "Case", "Convert all letters to lowercase.", None, |i, _| Ok(i.to_lowercase())),
        op("case-title", "Title Case", "Case", "Capitalize The First Letter Of Every Word.", None, |i, _| Ok(title_case(i))),
        op("case-sentence", "Sentence case", "Case", "Capitalize the first letter of each sentence.", None, |i, _| Ok(sentence_case(i))),
        op("case-capitalize", "Capitalize Words", "Case", "Capitalize each word, lower the rest.", None, |i, _| Ok(i.split_whitespace().map(cap_word).collect::<Vec<_>>().join(" "))),
        op("case-camel", "camelCase", "Case", "Convert words / snake / kebab text to camelCase.", None, |i, _| Ok(camel(i, false))),
        op("case-pascal", "PascalCase", "Case", "Convert text to PascalCase.", None, |i, _| Ok(camel(i, true))),
        op("case-snake", "snake_case", "Case", "Convert text to snake_case.", None, |i, _| Ok(words_of(i).join("_").to_lowercase())),
        op("case-kebab", "kebab-case", "Case", "Convert text to kebab-case.", None, |i, _| Ok(words_of(i).join("-").to_lowercase())),
        op("case-constant", "CONSTANT_CASE", "Case", "Convert text to CONSTANT_CASE.", None, |i, _| Ok(words_of(i).join("_").to_uppercase())),
        op("case-toggle", "tOGGLE cASE (invert)", "Case", "Swap upper and lower case for every letter.", None, |i, _| Ok(i.chars().map(|c| if c.is_uppercase() { c.to_lowercase().next().unwrap_or(c) } else if c.is_lowercase() { c.to_uppercase().next().unwrap_or(c) } else { c }).collect())),
        op("case-alternating", "aLtErNaTiNg CaSe", "Case", "Alternate upper/lower case letter by letter.", None, |i, _| { let mut up = false; Ok(i.chars().map(|c| { if c.is_alphabetic() { up = !up; if up { c.to_uppercase().next().unwrap_or(c) } else { c.to_lowercase().next().unwrap_or(c) } } else { c } }).collect()) }),
        // ---- Lines ----
        op("lines-sort-az", "Sort Lines A→Z", "Lines", "Sort lines alphabetically, ascending.", None, |i, _| sorted_lines(i, |a, b| a.to_lowercase().cmp(&b.to_lowercase()))),
        op("lines-sort-za", "Sort Lines Z→A", "Lines", "Sort lines alphabetically, descending.", None, |i, _| sorted_lines(i, |a, b| b.to_lowercase().cmp(&a.to_lowercase()))),
        op("lines-sort-length", "Sort Lines by Length", "Lines", "Shortest line first.", None, |i, _| sorted_lines(i, |a, b| a.chars().count().cmp(&b.chars().count()))),
        op("lines-sort-numeric", "Sort Lines Numerically", "Lines", "Sort lines as numbers, ascending.", None, |i, _| { let mut v: Vec<&str> = i.lines().collect(); v.sort_by(|a, b| a.trim().parse::<f64>().unwrap_or(f64::MAX).partial_cmp(&b.trim().parse::<f64>().unwrap_or(f64::MAX)).unwrap()); Ok(v.join("\n")) }),
        op("lines-reverse", "Reverse Line Order", "Lines", "Last line becomes first.", None, |i, _| Ok(i.lines().rev().collect::<Vec<_>>().join("\n"))),
        op("lines-shuffle", "Shuffle Lines", "Lines", "Randomize line order. Optional param: seed number.", Some("seed (optional)"), |i, p| { let seed = p.trim().parse::<u64>().unwrap_or_else(|_| Rng::from_time().0); let mut rng = Rng::new(seed); let mut v: Vec<&str> = i.lines().collect(); for k in (1..v.len()).rev() { let j = rng.below((k + 1) as u64) as usize; v.swap(k, j); } Ok(v.join("\n")) }),
        op("lines-dedupe", "Remove Duplicate Lines", "Lines", "Keep only the first copy of each line.", None, |i, _| { let mut seen = std::collections::HashSet::new(); Ok(i.lines().filter(|l| seen.insert(l.to_string())).collect::<Vec<_>>().join("\n")) }),
        op("lines-dedupe-ci", "Remove Duplicates (ignore case)", "Lines", "Remove duplicate lines ignoring upper/lower case.", None, |i, _| { let mut seen = std::collections::HashSet::new(); Ok(i.lines().filter(|l| seen.insert(l.to_lowercase())).collect::<Vec<_>>().join("\n")) }),
        op("lines-remove-empty", "Remove Empty Lines", "Lines", "Delete blank and whitespace-only lines.", None, |i, _| Ok(i.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n"))),
        op("lines-keep-containing", "Keep Lines Containing…", "Lines", "Keep only lines that contain the parameter text.", Some("text to keep"), |i, p| req(p).map(|p| i.lines().filter(|l| l.contains(p)).collect::<Vec<_>>().join("\n"))),
        op("lines-remove-containing", "Remove Lines Containing…", "Lines", "Delete lines that contain the parameter text.", Some("text to remove"), |i, p| req(p).map(|p| i.lines().filter(|l| !l.contains(p)).collect::<Vec<_>>().join("\n"))),
        op("lines-prefix-suffix", "Add Prefix / Suffix to Lines", "Lines", "Add text to the start and/or end of every line.", Some("prefix|||suffix"), |i, p| { let v = split_param(p); let pre = v.first().copied().unwrap_or(""); let suf = v.get(1).copied().unwrap_or(""); Ok(i.lines().map(|l| format!("{pre}{l}{suf}")).collect::<Vec<_>>().join("\n")) }),
        op("lines-number", "Add Line Numbers", "Lines", "Number each line starting at 1.", None, |i, _| Ok(i.lines().enumerate().map(|(n, l)| format!("{}. {}", n + 1, l)).collect::<Vec<_>>().join("\n"))),
        op("lines-unnumber", "Remove Line Numbers", "Lines", "Strip a leading '12. ' or '12) ' from each line.", None, |i, _| Ok(i.lines().map(strip_number).collect::<Vec<_>>().join("\n"))),
        op("lines-trim", "Trim Each Line", "Lines", "Remove spaces at the start and end of every line.", None, |i, _| Ok(i.lines().map(|l| l.trim()).collect::<Vec<_>>().join("\n"))),
        op("lines-wrap", "Wrap Text", "Lines", "Wrap text to a maximum line width.", Some("width in characters, e.g. 80"), |i, p| { let w = p.trim().parse::<usize>().map_err(|_| "Parameter must be a width like 80".to_string())?; if w < 4 { return Err("Width must be at least 4".into()); } Ok(wrap_text(i, w)) }),
        op("lines-unwrap", "Unwrap / Join Lines", "Lines", "Join wrapped lines back into paragraphs.", None, |i, _| Ok(unwrap_text(i))),
        op("words-sort", "Sort Words Alphabetically", "Lines", "Sort all words in the text A→Z, one per line.", None, |i, _| { let mut w: Vec<&str> = i.split_whitespace().collect(); w.sort_by_key(|a| a.to_lowercase()); Ok(w.join("\n")) }),
        op("words-dedupe", "Remove Duplicate Words", "Lines", "Keep each word once, in first-seen order.", None, |i, _| { let mut seen = std::collections::HashSet::new(); Ok(i.split_whitespace().filter(|w| seen.insert(w.to_lowercase())).collect::<Vec<_>>().join(" ")) }),
        // ---- Transform ----
        op("tx-reverse", "Reverse Text", "Transform", "Reverse all characters.", None, |i, _| Ok(i.chars().rev().collect())),
        op("tx-reverse-words", "Reverse Word Order", "Transform", "Reverse the order of words, keep each word readable.", None, |i, _| Ok(i.split_whitespace().rev().collect::<Vec<_>>().join(" "))),
        op("tx-reverse-each-word", "Reverse Each Word", "Transform", "Reverse letters inside each word, keep word order.", None, |i, _| Ok(i.split_inclusive(|c: char| c.is_whitespace()).map(|part| { let (w, s): (String, String) = part.chars().partition(|c| !c.is_whitespace()); let mut r: String = w.chars().rev().collect(); r.push_str(&s); r }).collect())),
        op("tx-repeat", "Repeat Text", "Transform", "Repeat the text N times.", Some("times, e.g. 3"), |i, p| { let n = p.trim().parse::<usize>().map_err(|_| "Parameter must be a count like 3".to_string())?; if n > 10000 { return Err("Max 10,000 repeats".into()); } Ok(i.repeat(n)) }),
        op("tx-remove-spaces", "Remove Extra Spaces", "Transform", "Collapse runs of spaces/tabs to one space, trim lines.", None, |i, _| Ok(i.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("\n"))),
        op("tx-remove-all-spaces", "Remove All Spaces", "Transform", "Delete every space, tab and line break.", None, |i, _| Ok(i.chars().filter(|c| !c.is_whitespace()).collect())),
        op("tx-remove-punctuation", "Remove Punctuation", "Transform", "Delete punctuation characters.", None, |i, _| Ok(i.chars().filter(|c| !c.is_ascii_punctuation()).collect())),
        op("tx-remove-numbers", "Remove Numbers", "Transform", "Delete all digits.", None, |i, _| Ok(i.chars().filter(|c| !c.is_ascii_digit()).collect())),
        op("tx-remove-vowels", "Remove Vowels", "Transform", "Delete a e i o u (both cases).", None, |i, _| Ok(i.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect())),
        op("tx-remove-accents", "Remove Accents / Diacritics", "Transform", "Strip common accented letters to their plain form (é→e, ñ→n…).", None, |i, _| Ok(i.chars().map(deaccent).collect())),
        op("tx-strip-html", "Strip HTML Tags", "Transform", "Remove all <tags>, keep the text.", None, |i, _| { let re = regex::Regex::new(r"<[^>]*>").unwrap(); Ok(re.replace_all(i, "").to_string()) }),
        op("tx-slugify", "Slugify (URL slug)", "Transform", "Make a web-address slug: lower-case-words-with-dashes.", None, |i, _| Ok(i.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect::<String>().split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-"))),
        op("tx-rot13", "ROT13", "Transform", "Rotate letters by 13 — run again to undo.", None, |i, _| Ok(rot_n(i, 13))),
        op("tx-caesar", "Caesar Cipher", "Transform", "Shift letters by N (negative to decode).", Some("shift, e.g. 3 or -3"), |i, p| { let n = p.trim().parse::<i32>().map_err(|_| "Parameter must be a shift like 3".to_string())?; Ok(rot_n(i, n.rem_euclid(26) as u8)) }),
        op("tx-pig-latin", "Pig Latin", "Transform", "Convert English words to Pig Latin.", None, |i, _| Ok(i.split_whitespace().map(pig).collect::<Vec<_>>().join(" "))),
        op("tx-upside-down", "Upside-Down Text", "Transform", "Flip text upside down using Unicode look-alikes.", None, |i, _| Ok(i.chars().rev().map(flip_char).collect())),
        op("tx-leet", "Leet Speak (1337)", "Transform", "Convert letters to leet-speak numbers/symbols.", None, |i, _| Ok(i.chars().map(leet).collect())),
        op("tx-morse-encode", "Morse Code Encode", "Transform", "Text → Morse code (· −).", None, |i, _| morse_encode(i)),
        op("tx-morse-decode", "Morse Code Decode", "Transform", "Morse code (letters split by spaces, words by /) → text.", None, |i, _| morse_decode(i)),
        op("tx-nato", "NATO Phonetic Alphabet", "Transform", "Spell text as Alfa, Bravo, Charlie…", None, |i, _| Ok(i.chars().filter(|c| c.is_alphanumeric()).map(nato).collect::<Vec<_>>().join(" "))),
        // ---- Analyse / extract ----
        op("an-count", "Count Characters / Words / Lines", "Analyse", "Full counts: characters, words, lines, sentences, paragraphs, reading & speaking time.", None, |i, _| Ok(count_report(i))),
        op("an-word-frequency", "Word Frequency", "Analyse", "How often each word appears, most frequent first.", None, |i, _| Ok(freq_report(i, false))),
        op("an-letter-frequency", "Letter Frequency", "Analyse", "How often each letter appears, most frequent first.", None, |i, _| Ok(freq_report(i, true))),
        op("an-find-replace", "Find and Replace", "Analyse", "Replace every occurrence of one text with another.", Some("find|||replace"), |i, p| { let v = split_param(p); if v.len() < 2 { return Err("Parameter must be: find|||replace".into()); } if v[0].is_empty() { return Err("Find text is empty".into()); } Ok(i.replace(v[0], v[1])) }),
        op("an-find-replace-regex", "Find and Replace (Regex)", "Analyse", "Find with a regular expression pattern, replace ($1 = first group).", Some("pattern|||replacement"), |i, p| { let v = split_param(p); if v.len() < 2 { return Err("Parameter must be: pattern|||replacement".into()); } let re = regex::Regex::new(v[0]).map_err(|e| format!("Bad pattern: {e}"))?; Ok(re.replace_all(i, v[1]).to_string()) }),
        op("an-grep", "Grep — Lines Matching a Pattern", "Analyse", "Keep only lines matching a regular expression.", Some("regex pattern"), |i, p| { let re = regex::Regex::new(req(p)?).map_err(|e| format!("Bad pattern: {e}"))?; Ok(i.lines().filter(|l| re.is_match(l)).collect::<Vec<_>>().join("\n")) }),
        op("an-extract-emails", "Extract Email Addresses", "Analyse", "Pull every email address out of the text, one per line.", None, |i, _| extract(i, r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")),
        op("an-extract-urls", "Extract URLs", "Analyse", "Pull every http/https web address out, one per line.", None, |i, _| extract(i, r#"https?://[^\s<>\"']+"#)),
        op("an-extract-numbers", "Extract Numbers", "Analyse", "Pull every number out of the text, one per line.", None, |i, _| extract(i, r"-?\d+(?:\.\d+)?")),
        op("an-diff", "Text Difference (Diff)", "Analyse", "Compare input text against the parameter text, line by line: − only in first, + only in second.", Some("second text"), diff),
        op("an-levenshtein", "Edit Distance (Levenshtein)", "Analyse", "How many single-character edits turn the input into the parameter text.", Some("other text"), |i, p| Ok(format!("Edit distance: {}", levenshtein(i, p)))),
        op("an-longest-shortest", "Longest / Shortest Lines & Words", "Analyse", "Report the longest and shortest lines and words.", None, |i, _| Ok(longest_shortest(i))),
    ]
}

fn req(p: &str) -> Result<&str, String> {
    let t = p.trim();
    if t.is_empty() {
        Err("This tool needs a Parameter — see the hint next to the Parameter box.".into())
    } else {
        Ok(t)
    }
}
fn cap_word(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
    }
}
fn words_of(s: &str) -> Vec<String> {
    // split camel humps and non-alphanumerics
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if c.is_uppercase() && prev_lower {
                out.push(std::mem::take(&mut cur));
            }
            prev_lower = c.is_lowercase();
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out.into_iter().filter(|w| !w.is_empty()).collect()
}
fn camel(s: &str, pascal: bool) -> String {
    words_of(s)
        .iter()
        .enumerate()
        .map(|(idx, w)| {
            let low = w.to_lowercase();
            if idx == 0 && !pascal {
                low
            } else {
                cap_word(&low)
            }
        })
        .collect()
}
fn title_case(s: &str) -> String {
    s.split_inclusive(|c: char| c.is_whitespace())
        .map(|part| {
            let (w, sp): (String, String) = part.chars().partition(|c| !c.is_whitespace());
            cap_word(&w) + &sp
        })
        .collect()
}
fn sentence_case(s: &str) -> String {
    let low = s.to_lowercase();
    let mut out = String::new();
    let mut cap = true;
    for c in low.chars() {
        if cap && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            cap = false;
        } else {
            out.push(c);
            if ".!?".contains(c) {
                cap = true;
            }
        }
    }
    out
}
fn sorted_lines<F: Fn(&str, &str) -> std::cmp::Ordering>(i: &str, f: F) -> Result<String, String> {
    let mut v: Vec<&str> = i.lines().collect();
    v.sort_by(|a, b| f(a, b));
    Ok(v.join("\n"))
}
fn strip_number(l: &str) -> String {
    let t = l.trim_start();
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        let rest = &t[digits.len()..];
        if let Some(r) = rest
            .strip_prefix(". ")
            .or_else(|| rest.strip_prefix(") "))
            .or_else(|| rest.strip_prefix(": "))
        {
            return r.to_string();
        }
    }
    l.to_string()
}
fn wrap_text(i: &str, w: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    for para in i.lines() {
        if para.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in para.split_whitespace() {
            if line.is_empty() {
                line = word.to_string();
            } else if line.chars().count() + 1 + word.chars().count() <= w {
                line.push(' ');
                line.push_str(word);
            } else {
                out.push(std::mem::take(&mut line));
                line = word.to_string();
            }
        }
        if !line.is_empty() {
            out.push(line);
        }
    }
    out.join("\n")
}
fn unwrap_text(i: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for l in i.lines() {
        if l.trim().is_empty() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(String::new());
        } else if cur.is_empty() {
            cur = l.trim().to_string();
        } else {
            cur.push(' ');
            cur.push_str(l.trim());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out.join("\n").trim().to_string()
}
fn rot_n(s: &str, n: u8) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                (((c as u8 - b'a') + n) % 26 + b'a') as char
            } else if c.is_ascii_uppercase() {
                (((c as u8 - b'A') + n) % 26 + b'A') as char
            } else {
                c
            }
        })
        .collect()
}
fn pig(w: &str) -> String {
    let lower = w.to_lowercase();
    let first = lower.chars().next().unwrap_or('a');
    if "aeiou".contains(first) {
        format!("{lower}way")
    } else {
        let rest: String = lower.chars().skip(1).collect();
        format!("{rest}{first}ay")
    }
}
fn deaccent(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => 'a',
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => 'A',
        'è' | 'é' | 'ê' | 'ë' | 'ē' => 'e',
        'È' | 'É' | 'Ê' | 'Ë' => 'E',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'Ì' | 'Í' | 'Î' | 'Ï' => 'I',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 'O',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ç' => 'c',
        'Ç' => 'C',
        'ý' | 'ÿ' => 'y',
        'ß' => 's',
        other => other,
    }
}
fn flip_char(c: char) -> char {
    match c {
        'a' => 'ɐ',
        'b' => 'q',
        'c' => 'ɔ',
        'd' => 'p',
        'e' => 'ǝ',
        'f' => 'ɟ',
        'g' => 'ƃ',
        'h' => 'ɥ',
        'i' => 'ᴉ',
        'j' => 'ɾ',
        'k' => 'ʞ',
        'l' => 'l',
        'm' => 'ɯ',
        'n' => 'u',
        'o' => 'o',
        'p' => 'd',
        'q' => 'b',
        'r' => 'ɹ',
        's' => 's',
        't' => 'ʇ',
        'u' => 'n',
        'v' => 'ʌ',
        'w' => 'ʍ',
        'x' => 'x',
        'y' => 'ʎ',
        'z' => 'z',
        '.' => '˙',
        '?' => '¿',
        '!' => '¡',
        other => other,
    }
}
fn leet(c: char) -> char {
    match c.to_ascii_lowercase() {
        'a' => '4',
        'e' => '3',
        'i' => '1',
        'o' => '0',
        's' => '5',
        't' => '7',
        'l' => '1',
        'g' => '9',
        'b' => '8',
        _ => c,
    }
}
const MORSE: &[(char, &str)] = &[
    ('a', ".-"),
    ('b', "-..."),
    ('c', "-.-."),
    ('d', "-.."),
    ('e', "."),
    ('f', "..-."),
    ('g', "--."),
    ('h', "...."),
    ('i', ".."),
    ('j', ".---"),
    ('k', "-.-"),
    ('l', ".-.."),
    ('m', "--"),
    ('n', "-."),
    ('o', "---"),
    ('p', ".--."),
    ('q', "--.-"),
    ('r', ".-."),
    ('s', "..."),
    ('t', "-"),
    ('u', "..-"),
    ('v', "...-"),
    ('w', ".--"),
    ('x', "-..-"),
    ('y', "-.--"),
    ('z', "--.."),
    ('0', "-----"),
    ('1', ".----"),
    ('2', "..---"),
    ('3', "...--"),
    ('4', "....-"),
    ('5', "....."),
    ('6', "-...."),
    ('7', "--..."),
    ('8', "---.."),
    ('9', "----."),
];
fn morse_encode(i: &str) -> Result<String, String> {
    let mut words = Vec::new();
    for w in i.split_whitespace() {
        let mut codes = Vec::new();
        for c in w.to_lowercase().chars() {
            match MORSE.iter().find(|(ch, _)| *ch == c) {
                Some((_, code)) => codes.push(*code),
                None => return Err(format!("No Morse code for '{c}'")),
            }
        }
        words.push(codes.join(" "));
    }
    Ok(words.join(" / "))
}
fn morse_decode(i: &str) -> Result<String, String> {
    let mut out = Vec::new();
    for w in i.split('/') {
        let mut word = String::new();
        for code in w.split_whitespace() {
            match MORSE.iter().find(|(_, c)| *c == code) {
                Some((ch, _)) => word.push(*ch),
                None => return Err(format!("Unknown Morse code '{code}'")),
            }
        }
        if !word.is_empty() {
            out.push(word);
        }
    }
    Ok(out.join(" "))
}
fn nato(c: char) -> String {
    match c.to_ascii_uppercase() {
        'A' => "Alfa",
        'B' => "Bravo",
        'C' => "Charlie",
        'D' => "Delta",
        'E' => "Echo",
        'F' => "Foxtrot",
        'G' => "Golf",
        'H' => "Hotel",
        'I' => "India",
        'J' => "Juliett",
        'K' => "Kilo",
        'L' => "Lima",
        'M' => "Mike",
        'N' => "November",
        'O' => "Oscar",
        'P' => "Papa",
        'Q' => "Quebec",
        'R' => "Romeo",
        'S' => "Sierra",
        'T' => "Tango",
        'U' => "Uniform",
        'V' => "Victor",
        'W' => "Whiskey",
        'X' => "X-ray",
        'Y' => "Yankee",
        'Z' => "Zulu",
        '0' => "Zero",
        '1' => "One",
        '2' => "Two",
        '3' => "Three",
        '4' => "Four",
        '5' => "Five",
        '6' => "Six",
        '7' => "Seven",
        '8' => "Eight",
        '9' => "Nine",
        _ => return c.to_string(),
    }
    .to_string()
}
fn count_report(i: &str) -> String {
    let chars = i.chars().count();
    let chars_no_spaces = i.chars().filter(|c| !c.is_whitespace()).count();
    let words = i.split_whitespace().count();
    let lines = if i.is_empty() { 0 } else { i.lines().count() };
    let sentences = i.matches(|c| ".!?".contains(c)).count();
    let paras = i.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    let read_min = words as f64 / 200.0;
    let speak_min = words as f64 / 130.0;
    format!("Characters: {chars}\nCharacters (no spaces): {chars_no_spaces}\nWords: {words}\nLines: {lines}\nSentences: {sentences}\nParagraphs: {paras}\nBytes (UTF-8): {}\nReading time: ~{read_min:.1} min (200 words/min)\nSpeaking time: ~{speak_min:.1} min (130 words/min)", i.len())
}
fn freq_report(i: &str, letters: bool) -> String {
    let mut map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    if letters {
        for c in i.to_lowercase().chars().filter(|c| c.is_alphabetic()) {
            *map.entry(c.to_string()).or_default() += 1;
        }
    } else {
        for w in i
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .filter(|w| !w.is_empty())
        {
            *map.entry(w.to_string()).or_default() += 1;
        }
    }
    let total: usize = map.values().sum();
    let mut v: Vec<(String, usize)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.iter()
        .map(|(k, n)| format!("{k}: {n} ({:.1}%)", *n as f64 / total.max(1) as f64 * 100.0))
        .collect::<Vec<_>>()
        .join("\n")
}
fn extract(i: &str, pat: &str) -> Result<String, String> {
    let re = regex::Regex::new(pat).unwrap();
    let found: Vec<&str> = re.find_iter(i).map(|m| m.as_str()).collect();
    if found.is_empty() {
        Ok("(none found)".into())
    } else {
        Ok(format!("Found {}:\n{}", found.len(), found.join("\n")))
    }
}
fn diff(a: &str, b: &str) -> Result<String, String> {
    // Simple LCS line diff
    let al: Vec<&str> = a.lines().collect();
    let bl: Vec<&str> = b.lines().collect();
    let (n, m) = (al.len(), bl.len());
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
    let mut out = Vec::new();
    let (mut x, mut y) = (0, 0);
    while x < n && y < m {
        if al[x] == bl[y] {
            out.push(format!("  {}", al[x]));
            x += 1;
            y += 1;
        } else if dp[x + 1][y] >= dp[x][y + 1] {
            out.push(format!("- {}", al[x]));
            x += 1;
        } else {
            out.push(format!("+ {}", bl[y]));
            y += 1;
        }
    }
    while x < n {
        out.push(format!("- {}", al[x]));
        x += 1;
    }
    while y < m {
        out.push(format!("+ {}", bl[y]));
        y += 1;
    }
    Ok(out.join("\n"))
}
fn levenshtein(a: &str, b: &str) -> usize {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=bv.len()).collect();
    let mut cur = vec![0; bv.len() + 1];
    for (x, ca) in av.iter().enumerate() {
        cur[0] = x + 1;
        for (y, cb) in bv.iter().enumerate() {
            cur[y + 1] = std::cmp::min(
                std::cmp::min(cur[y] + 1, prev[y + 1] + 1),
                prev[y] + if ca == cb { 0 } else { 1 },
            );
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[bv.len()]
}
fn longest_shortest(i: &str) -> String {
    let lines: Vec<&str> = i.lines().collect();
    let words: Vec<&str> = i.split_whitespace().collect();
    let by = |v: &[&str]| -> String {
        if v.is_empty() {
            return "—".into();
        }
        let longest = v.iter().max_by_key(|s| s.chars().count()).unwrap();
        let shortest = v.iter().min_by_key(|s| s.chars().count()).unwrap();
        format!(
            "longest ({} chars): {}\nshortest ({} chars): {}",
            longest.chars().count(),
            longest,
            shortest.chars().count(),
            shortest
        )
    };
    format!("Lines — {}\n\nWords — {}", by(&lines), by(&words))
}

// keep numbers_from referenced for future ops in this module (sort numeric uses parse directly)
#[allow(dead_code)]
fn _unused(i: &str) -> Result<Vec<f64>, String> {
    numbers_from(i)
}
