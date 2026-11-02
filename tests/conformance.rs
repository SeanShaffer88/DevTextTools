//! Conformance tests (v0.8.0): portable data folder mechanics in
//! the library, and the CLI maintenance surface end-to-end against
//! the real binary (`--version`, `--json`, `--clean`/`--uninstall`).

use devtexttools::paths;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("devtexttools-test-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn nan_in_user_numbers_never_panics_the_sorts() {
    // Rust's f64 parser accepts the literal "NaN"; sorting used
    // to unwrap partial_cmp, which is None for NaN — a panic on
    // pasted input. The sorts now use f64::total_cmp.
    let out = devtexttools::run_op("lines-sort-numeric", "3\nNaN\n1\n2", "")
        .expect("numeric sort survives NaN");
    assert!(out.lines().any(|l| l == "1"));
    let out = devtexttools::run_op("calc-stats", "1\nNaN\n2", "").expect("statistics survive NaN");
    assert!(out.contains("Count: 3"), "stats output: {out}");
}

#[test]
fn atomic_write_creates_and_overwrites_without_leftovers() {
    let dir = temp_dir("atomic");
    let file = dir.join("settings.txt");
    paths::atomic_write(&file, b"one").expect("first write");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "one");
    paths::atomic_write(&file, b"two").expect("overwrite");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "two");
    // No temp files are left behind by the rename-based write.
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn purge_candidates_lists_only_the_apps_own_files() {
    let dir = temp_dir("purge");
    std::fs::write(dir.join("settings.txt"), "s").unwrap();
    std::fs::write(dir.join("session.txt"), "s").unwrap();
    std::fs::write(dir.join("someone-elses.txt"), "s").unwrap();
    let mut names: Vec<String> = paths::purge_candidates(&dir)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["session.txt", "settings.txt"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Run the real binary with the given args, stdin and (optional)
/// data-dir override; returns (exit code, stdout, stderr).
fn run_bin(args: &[&str], stdin_text: &str, data_dir: Option<&PathBuf>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_DevTextTools"));
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = data_dir {
        cmd.env("DEVTEXTTOOLS_DATA_DIR", dir);
    }
    let mut child = cmd.spawn().expect("spawn DevTextTools");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin_text.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait for DevTextTools");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn cli_version_text_and_json() {
    let (code, stdout, _) = run_bin(&["--version"], "", None);
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        format!("DevTextTools {}", env!("CARGO_PKG_VERSION"))
    );
    let (code, stdout, _) = run_bin(&["--version", "--json"], "", None);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("version JSON parses");
    assert_eq!(v["ok"], true);
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn cli_tool_run_json_success_and_error() {
    let (code, stdout, _) = run_bin(&["case-upper", "--json"], "hello", None);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("run JSON parses");
    assert_eq!(v["ok"], true);
    assert_eq!(v["tool"], "case-upper");
    assert_eq!(v["output"], "HELLO");

    let (code, stdout, _) = run_bin(&["no-such-tool", "--json"], "x", None);
    assert_eq!(code, 1);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("error JSON parses");
    assert_eq!(v["ok"], false);
    assert!(v["error"].as_str().unwrap().contains("Unknown tool"));
}

#[test]
fn cli_clean_dry_run_silent_and_prompt_default() {
    let dir = temp_dir("clean");
    let settings = dir.join("settings.txt");
    std::fs::write(&settings, "fav\tx\n").unwrap();

    // Dry run reports but removes nothing.
    let (code, stdout, _) = run_bin(&["--clean", "--dry-run"], "", Some(&dir));
    assert_eq!(code, 0);
    assert!(stdout.contains("settings.txt"), "dry run lists the file");
    assert!(settings.exists(), "dry run removes nothing");

    // Interactive default: EOF at the prompt means No.
    let (code, _, _) = run_bin(&["--clean"], "", Some(&dir));
    assert_eq!(code, 0);
    assert!(settings.exists(), "unanswered prompt removes nothing");

    // A typed "y" confirms.
    let (code, _, _) = run_bin(&["--clean"], "y\n", Some(&dir));
    assert_eq!(code, 0);
    assert!(!settings.exists(), "confirmed clean removes the file");

    // Silent + JSON: one object out, file gone.
    std::fs::write(&settings, "fav\tx\n").unwrap();
    let (code, stdout, _) = run_bin(&["--uninstall", "--silent", "--json"], "", Some(&dir));
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("clean JSON parses");
    assert_eq!(v["ok"], true);
    assert_eq!(v["action"], "uninstall");
    assert!(!settings.exists(), "silent uninstall removes the file");

    // Unknown purge options are a usage error (exit 2).
    let (code, _, _) = run_bin(&["--clean", "--bogus"], "", Some(&dir));
    assert_eq!(code, 2);
    let _ = std::fs::remove_dir_all(&dir);
}
