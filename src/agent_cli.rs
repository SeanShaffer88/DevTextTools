//! Console-first command interface for scripts and local AI agents.
//! Requests are handled in memory through the same operation registry as the GUI.

use crate::{all_ops, paths, run_op, Op};
use serde_json::{json, Value};
use std::io::{self, Read, Write};

/// Run the console CLI and return its process exit code.
pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        print_help();
        return 0;
    }

    let (json_output, args) = split_json_flag(args);
    if args.is_empty() {
        print_help();
        return 0;
    }

    match args[0].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            0
        }
        "--version" | "version" => {
            if json_output {
                println!(
                    "{}",
                    json!({
                        "ok": true,
                        "name": "DevTextTools CLI",
                        "version": env!("CARGO_PKG_VERSION"),
                    })
                );
            } else {
                println!("DevTextTools CLI {}", env!("CARGO_PKG_VERSION"));
            }
            0
        }
        "--protocol" => {
            println!(
                "{}",
                json!({
                    "protocol": "devtexttools.tool-request.v1",
                    "request": {
                        "tool": "registered-tool-id",
                        "input": "text",
                        "param": "optional parameter text",
                    },
                    "response": {
                        "ok": "boolean",
                        "tool": "registered-tool-id",
                        "output": "text on success",
                        "error": "plain-language message on failure",
                    },
                    "transport": "one JSON request on stdin; one JSON response on stdout",
                    "state": "requests are not saved and no network calls are made",
                })
            );
            0
        }
        "--request-json" => {
            if args.len() != 1 {
                return emit_error(
                    true,
                    "Use --request-json by itself; send one JSON request on stdin.",
                    2,
                );
            }
            run_json_request()
        }
        "--clean" => run_purge(&args[1..], false, json_output),
        "--uninstall" => run_purge(&args[1..], true, json_output),
        "--list" | "list" => {
            if args.len() != 1 {
                return emit_error(json_output, "Usage: devtexttools-cli --list [--json]", 2);
            }
            print_catalog(&all_ops(), None, json_output);
            0
        }
        "--search" | "search" => {
            let query = args[1..].join(" ");
            print_catalog(&all_ops(), Some(&query), json_output);
            0
        }
        "--describe" | "describe" => {
            let Some(id) = args.get(1) else {
                return emit_error(
                    json_output,
                    "Usage: devtexttools-cli --describe <tool-id>",
                    2,
                );
            };
            let ops = all_ops();
            match ops.iter().find(|op| op.id == id) {
                Some(op) => {
                    let detail = tool_metadata(op);
                    if json_output {
                        println!("{}", json!({"ok": true, "tool": detail}));
                    } else {
                        print_tool(op);
                    }
                    0
                }
                None => emit_error(json_output, &format!("Unknown tool id '{id}'."), 1),
            }
        }
        "pdf-merge" => run_pdf_merge(&args[1..], json_output),
        "pdf-split" => run_pdf_split(&args[1..], json_output),
        "run" if args.len() > 1 => run_tool(&args[1], &args[2..], json_output),
        _ => run_tool(&args[0], &args[1..], json_output),
    }
}

fn split_json_flag(args: &[String]) -> (bool, Vec<String>) {
    let mut json_output = false;
    let mut rest = Vec::with_capacity(args.len());
    let mut after_param = false;
    for arg in args {
        if after_param {
            rest.push(arg.clone());
            after_param = false;
        } else if arg == "--param" {
            rest.push(arg.clone());
            after_param = true;
        } else if arg == "--json" {
            json_output = true;
        } else {
            rest.push(arg.clone());
        }
    }
    (json_output, rest)
}

fn print_catalog(ops: &[Op], query: Option<&str>, json_output: bool) {
    let query = query.unwrap_or_default().trim().to_lowercase();
    let matches: Vec<&Op> = ops
        .iter()
        .filter(|op| {
            query.is_empty()
                || op.id.to_lowercase().contains(&query)
                || op.name.to_lowercase().contains(&query)
                || op.category.to_lowercase().contains(&query)
                || op.description.to_lowercase().contains(&query)
        })
        .collect();

    if json_output {
        let tools: Vec<Value> = matches.iter().map(|op| tool_metadata(op)).collect();
        println!(
            "{}",
            json!({"ok": true, "count": tools.len(), "tools": tools})
        );
    } else {
        for op in &matches {
            println!("{:<28} [{}] {}", op.id, op.category, op.name);
        }
        if query.is_empty() {
            println!("\n{} tools total.", matches.len());
        } else {
            println!("\n{} tool(s) found.", matches.len());
        }
    }
}

fn tool_metadata(op: &Op) -> Value {
    json!({
        "id": op.id,
        "name": op.name,
        "category": op.category,
        "description": op.description,
        "param_hint": op.param_hint,
    })
}

fn print_tool(op: &Op) {
    println!("{} — {}", op.id, op.name);
    println!("Category: {}", op.category);
    println!("Description: {}", op.description);
    if let Some(hint) = op.param_hint {
        println!("Parameter: {hint}");
    }
}

fn run_json_request() -> i32 {
    let mut text = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut text) {
        return emit_error(true, &format!("Could not read request: {error}"), 2);
    }
    let request = match parse_request(&text) {
        Ok(request) => request,
        Err(error) => return emit_error(true, &error, 2),
    };
    match run_op(&request.tool, &request.input, &request.param) {
        Ok(output) => {
            println!(
                "{}",
                json!({"ok": true, "tool": request.tool, "output": output})
            );
            0
        }
        Err(error) => {
            println!(
                "{}",
                json!({"ok": false, "tool": request.tool, "error": error})
            );
            1
        }
    }
}

struct ToolRequest {
    tool: String,
    input: String,
    param: String,
}

fn parse_request(text: &str) -> Result<ToolRequest, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|error| format!("Invalid JSON request: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Request must be a JSON object.".to_string())?;
    let tool = object
        .get("tool")
        .or_else(|| object.get("id"))
        .and_then(Value::as_str)
        .filter(|tool| !tool.trim().is_empty())
        .ok_or_else(|| "Request needs a non-empty string field named 'tool'.".to_string())?;
    let input = request_text_field(object, "input")?;
    let param = request_text_field(object, "param")?;
    Ok(ToolRequest {
        tool: tool.trim().to_string(),
        input,
        param,
    })
}

fn request_text_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    match object.get(field) {
        None => Ok(String::new()),
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(format!("Request field '{field}' must be a string.")),
    }
}

fn run_tool(id: &str, args: &[String], json_output: bool) -> i32 {
    let mut param = String::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--param" => {
                let Some(value) = args.get(index + 1) else {
                    return emit_error(json_output, "--param needs a value.", 2);
                };
                param = value.clone();
                index += 2;
            }
            other => {
                return emit_error(
                    json_output,
                    &format!("Unknown option '{other}'. Use --help for usage."),
                    2,
                );
            }
        }
    }

    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        return emit_error(json_output, &format!("Could not read input: {error}"), 2);
    }
    match run_op(id, input.trim_end_matches(['\n', '\r']), &param) {
        Ok(output) if json_output => {
            println!("{}", json!({"ok": true, "tool": id, "output": output}));
            0
        }
        Ok(output) => {
            println!("{output}");
            0
        }
        Err(error) if json_output => {
            println!("{}", json!({"ok": false, "tool": id, "error": error}));
            1
        }
        Err(error) => emit_error(false, &error, 1),
    }
}

fn run_pdf_merge(args: &[String], json_output: bool) -> i32 {
    if args.len() < 3 {
        return emit_error(
            json_output,
            "Usage: devtexttools-cli pdf-merge <output.pdf> <input.pdf> <input.pdf> [...]",
            2,
        );
    }
    let output = std::path::PathBuf::from(&args[0]);
    let inputs: Vec<std::path::PathBuf> = args[1..].iter().map(std::path::PathBuf::from).collect();
    match crate::pdf::pdf_merge(&inputs, &output) {
        Ok(report) if json_output => {
            println!(
                "{}",
                json!({"ok": true, "command": "pdf-merge", "output": report})
            );
            0
        }
        Ok(report) => {
            println!("{report}");
            0
        }
        Err(error) if json_output => {
            println!(
                "{}",
                json!({"ok": false, "command": "pdf-merge", "error": error})
            );
            1
        }
        Err(error) => emit_error(false, &error, 1),
    }
}

fn run_pdf_split(args: &[String], json_output: bool) -> i32 {
    if args.is_empty() || args.len() > 2 {
        return emit_error(
            json_output,
            "Usage: devtexttools-cli pdf-split <input.pdf> [output-folder]",
            2,
        );
    }
    let input = std::path::PathBuf::from(&args[0]);
    let output_dir = args
        .get(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match crate::pdf::pdf_split(&input, &output_dir) {
        Ok(report) if json_output => {
            println!(
                "{}",
                json!({"ok": true, "command": "pdf-split", "output": report})
            );
            0
        }
        Ok(report) => {
            println!("{report}");
            0
        }
        Err(error) if json_output => {
            println!(
                "{}",
                json!({"ok": false, "command": "pdf-split", "error": error})
            );
            1
        }
        Err(error) => emit_error(false, &error, 1),
    }
}

fn run_purge(args: &[String], uninstall: bool, json_output: bool) -> i32 {
    let action = if uninstall { "uninstall" } else { "clean" };
    let mut dry_run = false;
    let mut silent = false;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--silent" => silent = true,
            other => {
                return emit_error(
                    json_output,
                    &format!("Unknown {action} option '{other}'."),
                    2,
                );
            }
        }
    }
    let fail = |message: &str| -> i32 {
        if json_output {
            println!(
                "{}",
                json!({"ok": false, "action": action, "error": message})
            );
            1
        } else {
            emit_error(false, message, 1)
        }
    };
    if crate::win32::another_instance_running() {
        return fail("DevTextTools is running — close the window first, then try again.");
    }

    let dir = paths::data_dir();
    let candidates = paths::purge_candidates(&dir);
    let names: Vec<String> = candidates
        .iter()
        .map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    if candidates.is_empty() {
        if json_output {
            println!(
                "{}",
                json!({
                    "ok": true,
                    "action": action,
                    "dry_run": dry_run,
                    "data_dir": dir.display().to_string(),
                    "removed": [],
                })
            );
        } else {
            println!(
                "Nothing to remove — no DevTextTools files in {}",
                dir.display()
            );
        }
        return 0;
    }
    if dry_run {
        if json_output {
            println!(
                "{}",
                json!({
                    "ok": true,
                    "action": action,
                    "dry_run": true,
                    "data_dir": dir.display().to_string(),
                    "would_remove": names,
                })
            );
        } else {
            println!(
                "Dry run — would remove {} file(s) from {}:",
                candidates.len(),
                dir.display()
            );
            for name in &names {
                println!("  {name}");
            }
        }
        return 0;
    }

    if !silent {
        eprintln!(
            "This removes {} DevTextTools file(s) from {}:",
            candidates.len(),
            dir.display()
        );
        for name in &names {
            eprintln!("  {name}");
        }
        eprint!("Continue? [y/N] ");
        let _ = io::stderr().flush();
        let mut answer = String::new();
        let _ = io::stdin().read_line(&mut answer);
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            if json_output {
                println!(
                    "{}",
                    json!({"ok": false, "action": action, "error": "not confirmed — nothing removed"})
                );
            } else {
                println!("Aborted — nothing removed.");
            }
            return 0;
        }
    }

    let mut removed = Vec::new();
    for (path, name) in candidates.iter().zip(names.iter()) {
        match std::fs::remove_file(path) {
            Ok(()) => removed.push(name.clone()),
            Err(error) => return fail(&format!("could not remove {name}: {error}")),
        }
    }
    if json_output {
        println!(
            "{}",
            json!({
                "ok": true,
                "action": action,
                "dry_run": false,
                "data_dir": dir.display().to_string(),
                "removed": removed,
            })
        );
    } else {
        println!("Removed {} file(s): {}", removed.len(), removed.join(", "));
        if uninstall {
            println!("DevTextTools' data is gone. Remove the program itself from Windows Settings or delete the portable folder.");
        }
    }
    0
}

fn emit_error(json_output: bool, message: &str, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", json!({"ok": false, "error": message}));
    } else {
        eprintln!("{message}");
    }
    exit_code
}

fn print_help() {
    println!(
        "DevTextTools CLI — offline tools for scripts and local AI.\n\n\
         --list [--json]                 list registered tools\n\
         --search <text> [--json]        search tool metadata\n\
         --describe <tool-id> [--json]   show one tool's metadata\n\
         --protocol                     print the JSON request protocol\n\
         --request-json                 read one JSON request from stdin\n\
         <tool-id> [--param <text>]      read tool input from stdin\n\
         pdf-merge <out.pdf> <...pdf>    merge PDF files\n\
         pdf-split <in.pdf> [folder]     split PDF pages into files\n\
         --clean [--dry-run|--silent]    remove the app's saved data\n\
         --uninstall [--dry-run|--silent] remove saved data and explain next steps\n\
         --version [--json]              print the version\n\
         --json                          return JSON for list, search, describe, run, and cleanup\n\
         --help                          show this help\n\n\
         JSON request: {{\"tool\":\"case-upper\",\"input\":\"hello\",\"param\":\"\"}}\n\
         JSON response: {{\"ok\":true,\"tool\":\"case-upper\",\"output\":\"HELLO\"}}\n\
         Requests run in memory; no network calls or input/history saving."
    );
}
