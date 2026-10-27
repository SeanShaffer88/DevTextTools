use super::Op;
use sha1::Digest as _;
use std::io::Read;

fn op(
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    run: fn(&str, &str) -> Result<String, String>,
) -> Op {
    Op {
        id,
        name,
        category: "Files",
        description: desc,
        param_hint: None,
        run,
    }
}

fn path_of(i: &str) -> Result<std::path::PathBuf, String> {
    let p = i.trim().trim_matches('"');
    if p.is_empty() {
        return Err(
            "Put the file's full path in the Input box, e.g. C:\\Users\\you\\file.txt".into(),
        );
    }
    let pb = std::path::PathBuf::from(p);
    if !pb.is_file() {
        return Err(format!("No file found at '{p}'"));
    }
    Ok(pb)
}

/// Feed a file through hash state in chunks; returns (md5, sha1, sha256, crc32).
pub fn hash_bytes_streaming(
    path: &std::path::Path,
) -> Result<(String, String, String, String), String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("Could not open the file: {e}"))?;
    let mut md5h = md5::Md5::new();
    let mut sha1 = sha1::Sha1::new();
    let mut sha256 = sha2::Sha256::new();
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut buf = [0u8; 65536];
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| format!("Could not read the file: {e}"))?;
        if n == 0 {
            break;
        }
        let chunk = &buf[..n];
        <md5::Md5 as md5::Digest>::update(&mut md5h, chunk);
        sha1::Digest::update(&mut sha1, chunk);
        sha2::Digest::update(&mut sha256, chunk);
        for &b in chunk {
            crc ^= b as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
    }
    Ok((
        hex(&<md5::Md5 as md5::Digest>::finalize(md5h)),
        hex(&sha1.finalize()),
        hex(&sha2::Digest::finalize(sha256)),
        format!("{:08x}", crc ^ 0xFFFF_FFFF),
    ))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn ops() -> Vec<Op> {
    vec![
        op("file-hash", "File Hashes (MD5 / SHA-1 / SHA-256 / CRC32)", "The Input is the file's PATH, not its contents. Works on files of any size (read in chunks). Use it to check a downloaded file against a published fingerprint.", |i, _| {
            let p = path_of(i)?;
            let (md5, sha1, sha256, crc) = hash_bytes_streaming(&p)?;
            Ok(format!("File: {}\nMD5: {md5}\nSHA-1: {sha1}\nSHA-256: {sha256}\nCRC32: {crc}", p.display()))
        }),
        op("file-info", "File Info (size, lines, words)", "The Input is the file's PATH. Reports size, and for text-like files the line, word and character counts.", |i, _| {
            let p = path_of(i)?;
            let meta = std::fs::metadata(&p).map_err(|e| e.to_string())?;
            let bytes = std::fs::read(&p).map_err(|e| format!("Could not read the file: {e}"))?;
            let text = String::from_utf8_lossy(&bytes);
            Ok(format!("File: {}\nSize: {} bytes\nLines: {}\nWords: {}\nCharacters: {}", p.display(), meta.len(), text.lines().count(), text.split_whitespace().count(), text.chars().count()))
        }),
        op("file-base64-encode", "File → Base64", "The Input is the file's PATH. Outputs the file's bytes as Base64 text (the way files are embedded in emails and web pages).", |i, _| {
            use base64::Engine;
            let p = path_of(i)?;
            let bytes = std::fs::read(&p).map_err(|e| format!("Could not read the file: {e}"))?;
            Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
        }),
        op("pdf-info", "PDF Info (pages, title, author)", "The Input is the PDF file's PATH. Reports the page count, PDF version, and the title/author saved inside the file.", |i, _| crate::pdf::pdf_info(i.trim().trim_matches('"'))),
        op("pdf-extract-text", "PDF → Text (extract)", "The Input is the PDF file's PATH. Pulls the readable text out of the PDF. Works on normal text PDFs; scanned-image PDFs have no text layer to extract (they need OCR).", |i, _| crate::pdf::pdf_extract_text(i.trim().trim_matches('"'))),
    ]
}
