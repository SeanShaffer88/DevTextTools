use super::dev::{csv_cell, parse_csv_line};
use super::Op;

fn op(
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: "Developer / Format",
        description: desc,
        param_hint: None,
        run,
    }
}

pub fn ops() -> Vec<Op> {
    vec![
        op("dev-yaml-to-json", "YAML → JSON (basic subset)", "Convert simple YAML (maps, lists, scalars — the everyday config-file kind) to JSON. Anchors, multi-line blocks and tags are not supported and are reported, not guessed.", |i, _| yaml_to_json(i)),
        op("dev-json-to-xml", "JSON → XML", "Convert JSON to simple XML: object keys become tags, arrays repeat their tag, the root tag is <root> (or the Parameter).", |i, p| json_to_xml(i, if p.trim().is_empty() { "root" } else { p.trim() })),
        op("dev-xml-to-json", "XML → JSON (basic)", "Convert simple XML to JSON: child tags become keys, repeated tags become arrays, attributes become @name keys, text becomes #text when mixed with children.", |i, _| xml_to_json(i)),
        op("dev-csv-to-tsv", "CSV → TSV", "Convert comma-separated values to tab-separated values.", |i, _| delim_convert(i, true)),
        op("dev-tsv-to-csv", "TSV → CSV", "Convert tab-separated values to comma-separated values (quoting values that need it).", |i, _| delim_convert(i, false)),
        op("dev-csv-to-md-table", "CSV → Markdown Table", "Turn CSV into a Markdown table (first row = header).", |i, _| csv_to_md(i)),
        op("dev-md-table-to-csv", "Markdown Table → CSV", "Turn a Markdown table back into CSV.", |i, _| md_to_csv(i)),
        op("dev-csv-transpose", "CSV Transpose (rows ↔ columns)", "Swap rows and columns of a CSV table.", |i, _| csv_transpose(i)),
    ]
}

fn rows_of(i: &str) -> Vec<Vec<String>> {
    i.lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse_csv_line)
        .collect()
}

fn delim_convert(i: &str, to_tsv: bool) -> Result<String, String> {
    if to_tsv {
        let rows = rows_of(i);
        Ok(rows
            .iter()
            .map(|r| {
                r.iter()
                    .map(|c| c.replace('\t', " "))
                    .collect::<Vec<_>>()
                    .join("\t")
            })
            .collect::<Vec<_>>()
            .join("\n"))
    } else {
        let rows: Vec<Vec<String>> = i
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split('\t').map(|s| s.to_string()).collect())
            .collect();
        Ok(rows
            .iter()
            .map(|r| r.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

fn csv_to_md(i: &str) -> Result<String, String> {
    let rows = rows_of(i);
    if rows.is_empty() {
        return Err("Empty CSV".into());
    }
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let cell =
        |r: &Vec<String>, k: usize| r.get(k).cloned().unwrap_or_default().replace('|', "\\|");
    let mut out = vec![format!(
        "| {} |",
        (0..width)
            .map(|k| cell(&rows[0], k))
            .collect::<Vec<_>>()
            .join(" | ")
    )];
    out.push(format!("| {} |", vec!["---"; width].join(" | ")));
    for r in &rows[1..] {
        out.push(format!(
            "| {} |",
            (0..width)
                .map(|k| cell(r, k))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    Ok(out.join("\n"))
}

fn md_to_csv(i: &str) -> Result<String, String> {
    let mut out = Vec::new();
    for line in i.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = t
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim().replace("\\|", "|"))
            .collect();
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
        {
            continue; // separator row
        }
        out.push(
            cells
                .iter()
                .map(|c| csv_cell(c))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if out.is_empty() {
        return Err("No Markdown table rows found (lines starting with |)".into());
    }
    Ok(out.join("\n"))
}

fn csv_transpose(i: &str) -> Result<String, String> {
    let rows = rows_of(i);
    if rows.is_empty() {
        return Err("Empty CSV".into());
    }
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut out = Vec::new();
    for col in 0..width {
        out.push(
            rows.iter()
                .map(|r| csv_cell(r.get(col).map(|s| s.as_str()).unwrap_or("")))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    Ok(out.join("\n"))
}

// ---------- JSON -> XML ----------
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn tag_name(k: &str) -> String {
    let mut out: String = k
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(true)
    {
        out = format!("_{out}");
    }
    out
}
fn json_xml(v: &serde_json::Value, tag: &str, out: &mut String) {
    match v {
        serde_json::Value::Object(m) => {
            out.push_str(&format!("<{tag}>"));
            for (k, val) in m {
                json_xml(val, &tag_name(k), out);
            }
            out.push_str(&format!("</{tag}>"));
        }
        serde_json::Value::Array(a) => {
            for item in a {
                json_xml(item, tag, out);
            }
        }
        serde_json::Value::Null => out.push_str(&format!("<{tag}/>")),
        other => {
            let s = match other {
                serde_json::Value::String(s) => s.clone(),
                x => x.to_string(),
            };
            out.push_str(&format!("<{tag}>{}</{tag}>", xml_escape(&s)));
        }
    }
}
fn json_to_xml(i: &str, root: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(i).map_err(|e| format!("Invalid JSON: {e}"))?;
    let mut out = String::new();
    json_xml(&v, &tag_name(root), &mut out);
    Ok(out)
}

// ---------- XML -> JSON (basic recursive parser) ----------
struct XmlParser<'a> {
    chars: Vec<char>,
    pos: usize,
    src: &'a str,
}
#[derive(Debug)]
enum Node {
    Elem {
        name: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
        text: String,
    },
}
impl<'a> XmlParser<'a> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    fn starts_with(&self, s: &str) -> bool {
        self.src[self.byte_pos()..].starts_with(s)
    }
    fn byte_pos(&self) -> usize {
        self.src
            .char_indices()
            .nth(self.pos)
            .map(|(b, _)| b)
            .unwrap_or(self.src.len())
    }
    fn bump(&mut self, n: usize) {
        self.pos += n;
    }
    fn skip_misc(&mut self) -> Result<(), String> {
        loop {
            while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                self.bump(1);
            }
            if self.starts_with("<!--") {
                let end = self.src[self.byte_pos()..]
                    .find("-->")
                    .ok_or("Unterminated XML comment")?;
                self.pos += self.src[..self.byte_pos() + end].chars().count() - self.pos + 3;
            } else if self.starts_with("<?") {
                let end = self.src[self.byte_pos()..]
                    .find("?>")
                    .ok_or("Unterminated <? … ?>")?;
                self.pos += self.src[..self.byte_pos() + end].chars().count() - self.pos + 2;
            } else if self.starts_with("<!") {
                let end = self.src[self.byte_pos()..]
                    .find('>')
                    .ok_or("Unterminated <! … >")?;
                self.pos += self.src[..self.byte_pos() + end].chars().count() - self.pos + 1;
            } else {
                return Ok(());
            }
        }
    }
    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .map(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':')
            .unwrap_or(false)
        {
            self.bump(1);
        }
        self.chars[start..self.pos].iter().collect()
    }
    fn read_until(&mut self, stop: char) -> String {
        let start = self.pos;
        while self.peek().map(|c| c != stop).unwrap_or(false) {
            self.bump(1);
        }
        self.chars[start..self.pos].iter().collect()
    }
    fn parse_element(&mut self) -> Result<Node, String> {
        if self.peek() != Some('<') {
            return Err(format!("Expected '<' at character {}", self.pos));
        }
        self.bump(1);
        let name = self.read_name();
        if name.is_empty() {
            return Err(format!("Missing tag name at character {}", self.pos));
        }
        let mut attrs = Vec::new();
        // attributes
        loop {
            while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                self.bump(1);
            }
            match self.peek() {
                Some('/') => {
                    self.bump(1);
                    if self.peek() != Some('>') {
                        return Err("Expected '>' after '/'".into());
                    }
                    self.bump(1);
                    return Ok(Node::Elem {
                        name,
                        attrs,
                        children: Vec::new(),
                        text: String::new(),
                    });
                }
                Some('>') => {
                    self.bump(1);
                    break;
                }
                Some(_) => {
                    let aname = self.read_name();
                    while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                        self.bump(1);
                    }
                    let mut aval = String::new();
                    if self.peek() == Some('=') {
                        self.bump(1);
                        while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                            self.bump(1);
                        }
                        let q = self.peek().ok_or("Unterminated attribute")?;
                        self.bump(1);
                        aval = self.read_until(q);
                        self.bump(1);
                    }
                    if !aname.is_empty() {
                        attrs.push((aname, aval));
                    }
                }
                None => return Err("XML ends inside a tag".into()),
            }
        }
        // content
        let mut children = Vec::new();
        let mut text = String::new();
        loop {
            if self.pos >= self.chars.len() {
                return Err(format!("XML ends before </{name}>"));
            }
            if self.starts_with(&format!("</{name}>"))
                || (self.starts_with("</") && {
                    // allow whitespace in close tag </name >
                    let save = self.pos;
                    self.bump(2);
                    let n = self.read_name();
                    let matched = n == name;
                    self.pos = save;
                    matched
                })
            {
                self.bump(2);
                let _ = self.read_name();
                while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                    self.bump(1);
                }
                if self.peek() != Some('>') {
                    return Err("Expected '>' in closing tag".into());
                }
                self.bump(1);
                break;
            } else if self.peek() == Some('<') {
                children.push(self.parse_element()?);
            } else {
                text.push_str(&self.read_until('<'));
            }
        }
        Ok(Node::Elem {
            name,
            attrs,
            children,
            text,
        })
    }
}
fn node_to_json(n: &Node) -> serde_json::Value {
    match n {
        Node::Elem {
            attrs,
            children,
            text,
            ..
        } => {
            let t = text.trim();
            if attrs.is_empty() && children.is_empty() {
                return serde_json::Value::String(xml_unescape(t));
            }
            let mut map = serde_json::Map::new();
            for (k, v) in attrs {
                map.insert(format!("@{k}"), serde_json::Value::String(xml_unescape(v)));
            }
            for child in children {
                {
                    let Node::Elem { name, .. } = child;
                    let jv = node_to_json(child);
                    match map.get_mut(name) {
                        Some(serde_json::Value::Array(a)) => a.push(jv),
                        Some(existing) => {
                            let first = existing.take();
                            map.insert(name.clone(), serde_json::Value::Array(vec![first, jv]));
                        }
                        None => {
                            map.insert(name.clone(), jv);
                        }
                    }
                }
            }
            if !t.is_empty() {
                map.insert("#text".into(), serde_json::Value::String(xml_unescape(t)));
            }
            serde_json::Value::Object(map)
        }
    }
}
fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
fn xml_to_json(i: &str) -> Result<String, String> {
    let mut p = XmlParser {
        chars: i.chars().collect(),
        pos: 0,
        src: i,
    };
    p.skip_misc()?;
    let root = p.parse_element()?;
    let mut map = serde_json::Map::new();
    let Node::Elem { name, .. } = &root;
    map.insert(name.clone(), node_to_json(&root));
    serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(|e| e.to_string())
}

// ---------- YAML -> JSON (basic subset) ----------
fn yaml_scalar(s: &str) -> serde_json::Value {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
    {
        return serde_json::Value::String(t[1..t.len() - 1].to_string());
    }
    match t {
        "null" | "~" | "" => return serde_json::Value::Null,
        "true" | "True" => return serde_json::Value::Bool(true),
        "false" | "False" => return serde_json::Value::Bool(false),
        _ => {}
    }
    if let Ok(n) = t.parse::<i64>() {
        return serde_json::Value::Number(n.into());
    }
    if let Ok(f) = t.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(n);
        }
    }
    serde_json::Value::String(t.to_string())
}

/// Parse lines of (indent, content) into a JSON value. Supports nested maps
/// by indentation and `- ` lists (items may be scalars or `key: value` maps).
fn yaml_block(
    lines: &[(usize, String)],
    pos: &mut usize,
    indent: usize,
) -> Result<serde_json::Value, String> {
    if *pos >= lines.len() {
        return Ok(serde_json::Value::Null);
    }
    if lines[*pos].1.starts_with("- ") || lines[*pos].1 == "-" {
        let mut arr = Vec::new();
        while *pos < lines.len() && lines[*pos].0 == indent && lines[*pos].1.starts_with('-') {
            let item_text = lines[*pos].1[1..].trim().to_string();
            if item_text.is_empty() {
                *pos += 1;
                let v = yaml_block(lines, pos, indent + 1)?;
                arr.push(v);
            } else if item_text.contains(": ") || item_text.ends_with(':') {
                // inline first key of a map item: synthesize a block
                let mut sub: Vec<(usize, String)> = vec![(indent + 1, item_text)];
                *pos += 1;
                while *pos < lines.len() && lines[*pos].0 > indent {
                    sub.push((lines[*pos].0, lines[*pos].1.clone()));
                    *pos += 1;
                }
                let mut sp = 0;
                let v = yaml_block(&sub, &mut sp, indent + 1)?;
                arr.push(v);
            } else {
                arr.push(yaml_scalar(&item_text));
                *pos += 1;
            }
        }
        return Ok(serde_json::Value::Array(arr));
    }
    let mut map = serde_json::Map::new();
    while *pos < lines.len() && lines[*pos].0 == indent && !lines[*pos].1.starts_with('-') {
        let line = lines[*pos].1.clone();
        let (key, rest) = line
            .split_once(':')
            .ok_or(format!("Expected 'key: value' but found '{line}'"))?;
        let key = key
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string();
        let rest = rest.trim();
        *pos += 1;
        if rest.is_empty() {
            if *pos < lines.len() && lines[*pos].0 > indent {
                let next_indent = lines[*pos].0;
                let v = yaml_block(lines, pos, next_indent)?;
                map.insert(key, v);
            } else {
                map.insert(key, serde_json::Value::Null);
            }
        } else {
            map.insert(key, yaml_scalar(rest));
        }
    }
    Ok(serde_json::Value::Object(map))
}
fn yaml_to_json(i: &str) -> Result<String, String> {
    if i.contains("\t") {
        return Err("YAML with tabs is not supported by this basic converter — replace tabs with spaces first".into());
    }
    for bad in ["&", "*", "|", ">", "<<:"] {
        if i.lines().any(|l| l.trim_start().starts_with(bad)) {
            return Err(format!(
                "This YAML uses a feature starting with '{bad}' (anchors/aliases/block text) that the basic converter does not support"
            ));
        }
    }
    let mut lines: Vec<(usize, String)> = Vec::new();
    for raw in i.lines() {
        let no_comment = match raw.find(" #") {
            Some(p) if !raw[..p].contains('"') && !raw[..p].contains('\'') => &raw[..p],
            _ => raw,
        };
        let t = no_comment.trim_end();
        if t.trim().is_empty() || t.trim() == "---" {
            continue;
        }
        let indent = t.len() - t.trim_start().len();
        lines.push((indent, t.trim_start().to_string()));
    }
    if lines.is_empty() {
        return Err("No YAML content found".into());
    }
    let base = lines[0].0;
    let mut pos = 0;
    let v = yaml_block(&lines, &mut pos, base)?;
    serde_json::to_string_pretty(&v).map_err(|e| e.to_string())
}
