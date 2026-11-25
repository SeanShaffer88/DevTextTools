use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn invoke(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_devtexttools-cli"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start CLI process");
    child
        .stdin
        .take()
        .expect("CLI stdin")
        .write_all(stdin.as_bytes())
        .expect("write CLI input");
    child.wait_with_output().expect("wait for CLI process")
}

#[test]
fn json_catalog_includes_every_registered_operation() {
    let output = invoke(&["--list", "--json"], "");
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).expect("valid catalog JSON");
    let tools = response["tools"].as_array().expect("tools array");
    assert_eq!(
        response["count"].as_u64(),
        Some(tools.len() as u64),
        "count matches the returned tools"
    );
    assert_eq!(tools.len(), devtexttools::all_ops().len());
    assert!(tools.iter().any(|tool| tool["id"] == "case-upper"));
    assert!(tools.iter().all(|tool| {
        tool["id"].is_string()
            && tool["name"].is_string()
            && tool["category"].is_string()
            && tool["description"].is_string()
    }));
}

#[test]
fn json_request_runs_a_tool_and_returns_one_json_object() {
    let output = invoke(
        &["--request-json"],
        r#"{"tool":"case-upper","input":"hello","param":""}"#,
    );
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).expect("valid response JSON");
    assert_eq!(response["ok"], true);
    assert_eq!(response["tool"], "case-upper");
    assert_eq!(response["output"], "HELLO");
}

#[test]
fn malformed_json_returns_structured_error_and_nonzero_exit() {
    let output = invoke(&["--request-json"], "{");
    assert_eq!(output.status.code(), Some(2));
    let response: Value = serde_json::from_slice(&output.stdout).expect("valid error JSON");
    assert_eq!(response["ok"], false);
    assert!(response["error"].as_str().is_some());
}

#[test]
fn unknown_tool_returns_structured_error_and_nonzero_exit() {
    let output = invoke(
        &["--request-json"],
        r#"{"tool":"not-a-real-tool","input":"hello"}"#,
    );
    assert_eq!(output.status.code(), Some(1));
    let response: Value = serde_json::from_slice(&output.stdout).expect("valid error JSON");
    assert_eq!(response["ok"], false);
    assert_eq!(response["tool"], "not-a-real-tool");
    assert!(response["error"].as_str().is_some());
}
