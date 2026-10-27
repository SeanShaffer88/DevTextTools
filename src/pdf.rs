//! PDF jobs built on lopdf: info, text extraction, merge, split.
//! All functions take file paths and return a plain-language report or the
//! extracted text. They are called identically from the operation registry
//! (info/extract), the CLI (all four), and the GUI file buttons (all four) —
//! no surface gets its own private implementation.

use lopdf::{Document, Object};
use std::path::{Path, PathBuf};

fn load(path: &str) -> Result<Document, String> {
    if !Path::new(path).is_file() {
        return Err(format!("No file found at '{path}'"));
    }
    Document::load(path).map_err(|e| {
        format!("Could not read that PDF ({e}). It may be damaged, or password-protected — protected PDFs are not supported.")
    })
}

fn pdf_string(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16BE
        let units: Vec<u16> = bytes[2..]
            .chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| ((c[0] as u16) << 8) | c[1] as u16)
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        bytes.iter().map(|&b| b as char).collect()
    }
}

fn info_field(doc: &Document, key: &[u8]) -> Option<String> {
    let info_ref = doc.trailer.get(b"Info").ok()?.as_reference().ok()?;
    let dict = doc.get_dictionary(info_ref).ok()?;
    match dict.get(key) {
        Ok(Object::String(bytes, _)) => {
            let s = pdf_string(bytes);
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }
        _ => None,
    }
}

/// Page count, PDF version, and the document info saved inside the file.
pub fn pdf_info(path: &str) -> Result<String, String> {
    let doc = load(path)?;
    let pages = doc.get_pages().len();
    let mut out = format!("File: {path}\nPDF version: {}\nPages: {pages}", doc.version);
    if let Some(t) = info_field(&doc, b"Title") {
        out.push_str(&format!("\nTitle: {t}"));
    }
    if let Some(a) = info_field(&doc, b"Author") {
        out.push_str(&format!("\nAuthor: {a}"));
    }
    if let Some(c) = info_field(&doc, b"Creator") {
        out.push_str(&format!("\nCreated by: {c}"));
    }
    Ok(out)
}

/// Extract the text layer of a PDF, page by page.
pub fn pdf_extract_text(path: &str) -> Result<String, String> {
    let doc = load(path)?;
    let page_numbers: Vec<u32> = doc.get_pages().keys().copied().collect();
    if page_numbers.is_empty() {
        return Err("That PDF has no pages".into());
    }
    let chunks = doc.extract_text_chunks(&page_numbers);
    let mut out = String::new();
    for (idx, chunk) in chunks.iter().enumerate() {
        match chunk {
            Ok(text) => {
                if page_numbers.len() > 1 {
                    out.push_str(&format!("----- Page {} -----\n", idx + 1));
                }
                out.push_str(text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            Err(e) => {
                return Err(format!("Could not extract text from page {}: {e}", idx + 1));
            }
        }
    }
    if out.trim().is_empty() {
        return Err("No text layer found — this looks like a scanned-image PDF. Scanned pages need OCR (optical character recognition), which DevTextTools does not do.".into());
    }
    Ok(out)
}

/// Merge several PDFs into one, in the order given. Page resources/media
/// boxes declared on a source's page-tree root are copied onto each page
/// that does not declare its own, so inherited settings still apply.
pub fn pdf_merge(inputs: &[PathBuf], output: &Path) -> Result<String, String> {
    if inputs.len() < 2 {
        return Err("Merging needs at least two PDF files".into());
    }
    let mut merged = Document::with_version("1.7");
    let pages_id = merged.new_object_id();
    let mut kids: Vec<Object> = Vec::new();
    let mut total: u32 = 0;

    for input in inputs {
        let mut doc = load(&input.to_string_lossy())?;
        // Renumber into a non-overlapping id range first; lopdf updates all
        // internal references as it goes. Shared settings are read AFTER
        // renumbering so their references are already consistent.
        let offset = merged.max_id + 1;
        doc.renumber_objects_with(offset);
        merged.max_id = doc.max_id;
        let mut shared: Vec<(Vec<u8>, Object)> = Vec::new();
        if let Ok(catalog) = doc.catalog() {
            if let Ok(pages_ref) = catalog.get(b"Pages").and_then(Object::as_reference) {
                if let Ok(tree) = doc.get_dictionary(pages_ref) {
                    for key in [
                        b"Resources".as_slice(),
                        b"MediaBox".as_slice(),
                        b"CropBox".as_slice(),
                    ] {
                        if let Ok(v) = tree.get(key) {
                            shared.push((key.to_vec(), v.clone()));
                        }
                    }
                }
            }
        }

        let src_pages = doc.get_pages();
        total += src_pages.len() as u32;
        for obj_id in src_pages.values() {
            kids.push(Object::Reference(*obj_id));
        }
        for (obj_id, mut obj) in doc.objects.into_iter() {
            if let Object::Dictionary(ref mut d) = obj {
                let typ = d
                    .get(b"Type")
                    .and_then(Object::as_name)
                    .unwrap_or(b"")
                    .to_vec();
                if typ == b"Catalog" || typ == b"Pages" {
                    continue;
                }
                if typ == b"Page" {
                    d.set("Parent", Object::Reference(pages_id));
                    for (k, v) in &shared {
                        if d.get(k.as_slice()).is_err() {
                            d.set(k.clone(), v.clone());
                        }
                    }
                }
            }
            merged.objects.insert(obj_id, obj);
        }
    }

    let mut pages_dict = lopdf::Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Kids", Object::Array(kids));
    pages_dict.set("Count", Object::Integer(total as i64));
    merged
        .objects
        .insert(pages_id, Object::Dictionary(pages_dict));
    let mut catalog_dict = lopdf::Dictionary::new();
    catalog_dict.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog_dict.set("Pages", Object::Reference(pages_id));
    let catalog_id = merged.add_object(Object::Dictionary(catalog_dict));
    merged.trailer.set("Root", Object::Reference(catalog_id));
    merged
        .save(output)
        .map_err(|e| format!("Could not write the merged PDF: {e}"))?;
    Ok(format!(
        "Merged {} files into {} ({total} pages)",
        inputs.len(),
        output.display()
    ))
}

/// Split a PDF into one file per page in `out_dir`. Files are named
/// `<name>-page-1.pdf`, `-page-2.pdf`, …
pub fn pdf_split(input: &Path, out_dir: &Path) -> Result<String, String> {
    let doc = load(&input.to_string_lossy())?;
    let page_count = doc.get_pages().len();
    if page_count == 0 {
        return Err("That PDF has no pages".into());
    }
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("Could not create the output folder: {e}"))?;
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());
    let mut written = Vec::new();
    for page in 1..=(page_count as u32) {
        let mut one = load(&input.to_string_lossy())?;
        let others: Vec<u32> = (1..=(page_count as u32)).filter(|&p| p != page).collect();
        one.delete_pages(&others);
        one.compress();
        let out_path = out_dir.join(format!("{stem}-page-{page}.pdf"));
        one.save(&out_path)
            .map_err(|e| format!("Could not write {}: {e}", out_path.display()))?;
        written.push(out_path.display().to_string());
    }
    Ok(format!(
        "Split into {page_count} files:\n{}",
        written.join("\n")
    ))
}
