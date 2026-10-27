use devtexttools::{all_ops, run_op};

#[test]
fn registry_is_large_and_unique() {
    let ops = all_ops();
    assert!(ops.len() >= 120, "expected 120+ tools, got {}", ops.len());
    let mut ids: Vec<&str> = ops.iter().map(|o| o.id).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(n, ids.len(), "duplicate tool ids");
}

#[test]
fn case_tools() {
    assert_eq!(
        run_op("case-upper", "hello World", "").unwrap(),
        "HELLO WORLD"
    );
    assert_eq!(
        run_op("case-snake", "helloWorld now", "").unwrap(),
        "hello_world_now"
    );
    assert_eq!(
        run_op("case-camel", "hello_world-now", "").unwrap(),
        "helloWorldNow"
    );
    assert_eq!(
        run_op("case-title", "hello world", "").unwrap(),
        "Hello World"
    );
}

#[test]
fn line_tools() {
    assert_eq!(run_op("lines-sort-az", "b\na\nc", "").unwrap(), "a\nb\nc");
    assert_eq!(run_op("lines-dedupe", "a\na\nb", "").unwrap(), "a\nb");
    assert_eq!(
        run_op("lines-prefix-suffix", "x", "> |||<").unwrap(),
        "> x<"
    );
}

#[test]
fn encode_roundtrips() {
    let b64 = run_op("enc-base64-encode", "hello", "").unwrap();
    assert_eq!(b64, "aGVsbG8=");
    assert_eq!(run_op("enc-base64-decode", &b64, "").unwrap(), "hello");
    let hex = run_op("enc-hex-encode", "AB", "").unwrap();
    assert_eq!(hex, "4142");
    assert_eq!(run_op("enc-hex-decode", &hex, "").unwrap(), "AB");
    let b32 = run_op("enc-base32-encode", "hi", "").unwrap();
    assert_eq!(run_op("enc-base32-decode", &b32, "").unwrap(), "hi");
    assert_eq!(run_op("tx-morse-encode", "SOS", "").unwrap(), "... --- ...");
    assert_eq!(run_op("tx-morse-decode", "... --- ...", "").unwrap(), "sos");
}

#[test]
fn hashes_known_vectors() {
    assert_eq!(
        run_op("hash-md5", "abc", "").unwrap(),
        "900150983cd24fb0d6963f7d28e17f72"
    );
    assert_eq!(
        run_op("hash-sha256", "abc", "").unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        run_op("hash-sha1", "abc", "").unwrap(),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(run_op("hash-crc32", "123456789", "").unwrap(), "cbf43926");
    // RFC 4231 test case 1: key = 20 x 0x0b, data = "Hi There"
    let key = "\u{b}".repeat(20);
    assert_eq!(
        run_op("hash-hmac-sha256", "Hi There", &key).unwrap(),
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
    );
}

#[test]
fn dev_tools() {
    assert_eq!(
        run_op("dev-json-minify", "{ \"a\": 1 }", "").unwrap(),
        "{\"a\":1}"
    );
    assert!(run_op("dev-json-pretty", "[1,2]", "")
        .unwrap()
        .contains('\n'));
    let csv = run_op("dev-json-to-csv", "[{\"a\":1,\"b\":2}]", "").unwrap();
    assert_eq!(csv, "a,b\n1,2");
    let json = run_op("dev-csv-to-json", "a,b\n1,2", "").unwrap();
    assert!(json.contains("\"a\": \"1\""));
    let jwt = format!("{}.{}.sig", "eyJhbGciOiJIUzI1NiJ9", "eyJzdWIiOiIxMjMifQ");
    assert!(run_op("dev-jwt-decode", &jwt, "").unwrap().contains("123"));
    assert!(run_op("dev-json-validate", "not json", "").is_err());
}

#[test]
fn converters() {
    assert_eq!(run_op("conv-base", "255", "10|||16").unwrap(), "FF");
    assert_eq!(run_op("conv-int-to-roman", "14", "").unwrap(), "XIV");
    assert_eq!(run_op("conv-roman-to-int", "XIV", "").unwrap(), "14");
    let len = run_op("conv-length", "1", "mile|||km").unwrap();
    assert!(len.contains("1.609344"), "got {len}");
    assert_eq!(
        run_op("conv-temp", "100", "C|||F").unwrap(),
        "100 C = 212 F"
    );
    assert_eq!(
        run_op("conv-timestamp-to-date", "0", "").unwrap(),
        "0 = 1970-01-01 00:00:00 UTC"
    );
    assert_eq!(
        run_op("conv-date-to-timestamp", "1970-01-02", "").unwrap(),
        "86400"
    );
    assert_eq!(
        run_op("conv-color-rgb-hex", "229,72,77", "").unwrap(),
        "#E5484D"
    );
}

#[test]
fn calculators() {
    assert_eq!(run_op("calc-expression", "2*(3+4)^2", "").unwrap(), "98");
    assert_eq!(run_op("calc-expression", "sqrt(16)+1", "").unwrap(), "5");
    assert_eq!(run_op("calc-factorial", "5", "").unwrap(), "5! = 120");
    assert_eq!(
        run_op("calc-fibonacci", "10", "").unwrap(),
        "Fibonacci(10) = 55"
    );
    assert!(run_op("calc-subnet", "192.168.1.10/24", "")
        .unwrap()
        .contains("192.168.1.0"));
    assert_eq!(
        run_op("an-levenshtein", "kitten", "sitting").unwrap(),
        "Edit distance: 3"
    );
}

#[test]
fn generators_produce_shapes() {
    let u = run_op("gen-uuid", "", "3").unwrap();
    assert_eq!(u.lines().count(), 3);
    assert_eq!(u.lines().next().unwrap().len(), 36);
    let pw = run_op("gen-password", "", "16|||2").unwrap();
    assert_eq!(pw.lines().count(), 2);
    assert!(pw.lines().all(|l| l.chars().count() == 16));
}

#[test]
fn errors_are_plain_and_specific() {
    assert!(run_op("nope", "", "").unwrap_err().contains("Unknown tool"));
    assert!(run_op("enc-base64-decode", "!!!", "").is_err());
    assert!(run_op("lines-wrap", "hello", "").is_err());
}

#[test]
fn v0_2_fancy_and_vigenere() {
    assert_eq!(run_op("fancy-bold", "Ab1", "").unwrap(), "𝐀𝐛𝟏");
    assert_eq!(run_op("fancy-circled", "A0", "").unwrap(), "Ⓐ⓪");
    assert_eq!(run_op("fancy-small-caps", "Hi", "").unwrap(), "ʜɪ");
    assert!(run_op("fancy-strikethrough", "ab", "")
        .unwrap()
        .contains('\u{336}'));
    // Vigenere known answer: key KEY, plaintext HELLO -> RIJVS
    assert_eq!(
        run_op("tx-vigenere-encode", "HELLO", "KEY").unwrap(),
        "RIJVS"
    );
    assert_eq!(
        run_op("tx-vigenere-decode", "RIJVS", "KEY").unwrap(),
        "HELLO"
    );
}

#[test]
fn v0_2_data_conversions() {
    let yaml = "name: test\nport: 8080\nflags:\n  debug: true\nlist:\n  - one\n  - two";
    let j = run_op("dev-yaml-to-json", yaml, "").unwrap();
    assert!(j.contains("\"name\": \"test\""), "{j}");
    assert!(j.contains("\"debug\": true"), "{j}");
    assert!(j.contains("\"one\""), "{j}");
    let xml = run_op("dev-json-to-xml", "{\"a\":1,\"b\":\"x\"}", "").unwrap();
    assert!(
        xml.contains("<a>1</a>") && xml.contains("<b>x</b>"),
        "{xml}"
    );
    let back = run_op(
        "dev-xml-to-json",
        "<root><a>1</a><a>2</a><b x=\"y\">t</b></root>",
        "",
    )
    .unwrap();
    assert!(back.contains("\"a\""), "{back}");
    assert!(back.contains("\"@x\": \"y\""), "{back}");
    assert_eq!(
        run_op("dev-csv-to-tsv", "a,b\n1,2", "").unwrap(),
        "a\tb\n1\t2"
    );
    assert_eq!(
        run_op(
            "dev-md-table-to-csv",
            "| a | b |\n| --- | --- |\n| 1 | 2 |",
            ""
        )
        .unwrap(),
        "a,b\n1,2"
    );
    assert_eq!(
        run_op("dev-csv-transpose", "a,b\n1,2", "").unwrap(),
        "a,1\nb,2"
    );
}

#[test]
fn v0_2_calculators_and_dates() {
    assert!(run_op("calc-circle", "1", "")
        .unwrap()
        .contains("Area: 3.14159"));
    assert!(run_op("calc-quadratic", "1|||-3|||2", "")
        .unwrap()
        .contains("x1 = 2"));
    // Ohm's law: V=12, R=4 -> I=3, P=36
    let o = run_op("calc-ohms", "V=12|||R=4", "").unwrap();
    assert!(o.contains("3 amps") && o.contains("36 watts"), "{o}");
    let g = run_op("calc-gpa", "A|||3\nB|||4", "").unwrap();
    assert!(g.contains("GPA: 3.43"), "{g}");
    assert_eq!(
        run_op("date-weekday", "1970-01-01", "").unwrap(),
        "1970-01-01 is a Thursday"
    );
    assert_eq!(
        run_op("date-weekday", "2026-10-25", "").unwrap(),
        "2026-10-25 is a Sunday"
    );
    assert!(run_op("date-add-days", "2026-01-31", "1")
        .unwrap()
        .contains("2026-02-01"));
    assert_eq!(
        run_op("date-days-in-month", "2024-02", "").unwrap(),
        "2024-02 has 29 days"
    );
    assert_eq!(
        run_op("date-days-in-month", "2025-02", "").unwrap(),
        "2025-02 has 28 days"
    );
    // 2026-10-23 (Fri) -> 2026-10-26 (Mon): business days = 1 (Mon), calendar = 3
    let bd = run_op("date-business-days", "2026-10-23", "2026-10-26").unwrap();
    assert!(bd.contains("Business days: 1"), "{bd}");
    let wk = run_op("date-week-number", "2026-01-01", "").unwrap();
    assert!(wk.contains("2026-W01"), "{wk}");
    assert_eq!(devtexttools::fmt_clock(3661), "01:01:01");
}

#[test]
#[allow(unused_mut)]
fn v0_3_files_and_pdf() {
    use lopdf::content::{Content, Operation};
    use lopdf::{Document, Object, Stream};
    macro_rules! dictionary { ($($k:expr => $v:expr),* $(,)?) => {{ let mut d = lopdf::Dictionary::new(); $(d.set($k, $v);)* d }}; }

    let dir = std::env::temp_dir().join("devtexttools-test");
    std::fs::create_dir_all(&dir).unwrap();

    // file ops
    let txt = dir.join("note.txt");
    std::fs::write(&txt, "hello file\nsecond line\n").unwrap();
    let h = run_op("file-hash", txt.to_str().unwrap(), "").unwrap();
    assert!(h.contains("SHA-256:"), "{h}");
    let info = run_op("file-info", txt.to_str().unwrap(), "").unwrap();
    assert!(info.contains("Lines: 2"), "{info}");
    let b64 = run_op("file-base64-encode", txt.to_str().unwrap(), "").unwrap();
    assert_eq!(
        run_op("enc-base64-decode", &b64, "").unwrap(),
        "hello file\nsecond line\n"
    );

    // build a real one-page PDF with lopdf and a visible text line
    fn make_pdf(path: &std::path::Path, text: &str) {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => Object::Reference(font_id) },
        });
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 24.into()]),
                Operation::new("Td", vec![100.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal(text)]),
                Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "Contents" => Object::Reference(content_id),
            "Resources" => Object::Reference(resources_id),
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![Object::Reference(page_id)],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        });
        doc.trailer.set("Root", Object::Reference(catalog_id));
        doc.save(path).unwrap();
    }

    let p1 = dir.join("one.pdf");
    let p2 = dir.join("two.pdf");
    make_pdf(&p1, "Hello PDF World");
    make_pdf(&p2, "Second Document");

    let pinfo = run_op("pdf-info", p1.to_str().unwrap(), "").unwrap();
    assert!(pinfo.contains("Pages: 1"), "{pinfo}");
    let extracted = run_op("pdf-extract-text", p1.to_str().unwrap(), "").unwrap();
    assert!(extracted.contains("Hello PDF World"), "{extracted}");

    // merge -> 2 pages, both texts extractable
    let merged = dir.join("merged.pdf");
    let report = devtexttools::pdf::pdf_merge(&[p1.clone(), p2.clone()], &merged).unwrap();
    assert!(report.contains("2 pages"), "{report}");
    let minfo = devtexttools::pdf::pdf_info(&merged.to_string_lossy()).unwrap();
    assert!(minfo.contains("Pages: 2"), "{minfo}");
    let mtext = devtexttools::pdf::pdf_extract_text(&merged.to_string_lossy()).unwrap();
    assert!(
        mtext.contains("Hello PDF World") && mtext.contains("Second Document"),
        "{mtext}"
    );

    // split -> 2 single-page files
    let outdir = dir.join("split");
    let sreport = devtexttools::pdf::pdf_split(&merged, &outdir).unwrap();
    assert!(sreport.contains("Split into 2 files"), "{sreport}");
    let s1 = outdir.join("merged-page-1.pdf");
    assert!(s1.is_file());
    let sinfo = devtexttools::pdf::pdf_info(&s1.to_string_lossy()).unwrap();
    assert!(sinfo.contains("Pages: 1"), "{sinfo}");
}
