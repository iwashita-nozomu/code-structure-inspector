use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_code-structure-inspector"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn stdio_handshake_tools_calls_and_notification_framing() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"fixture","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"inspect_repository","arguments":{"root":root}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"related_files","arguments":{"root":root,"path":"src/worker.rs"}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"inspect_repository","arguments":{"root":root,"shell":"touch leaked"}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"shell","arguments":{"root":root}}}),
    ];
    let input = requests
        .iter()
        .map(|r| format!("{r}\n"))
        .collect::<String>();
    let out = run(&input);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let messages: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(messages.len(), 7);
    assert_eq!(messages[0]["error"]["code"], -32000);
    assert_eq!(messages[1]["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(messages[2]["result"]["tools"].as_array().unwrap().len(), 2);
    assert_eq!(
        messages[2]["result"]["tools"][0]["annotations"]["readOnlyHint"],
        true
    );
    let report: Value = serde_json::from_str(
        messages[3]["result"]["content"][0]["text"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["analysis"], "syntactic_evidence");
    assert_eq!(messages[4]["result"]["isError"], false);
    assert_eq!(messages[5]["result"]["isError"], true);
    assert_eq!(messages[6]["result"]["isError"], true);
}
#[test]
fn malformed_and_oversized_requests_do_not_echo_input() {
    let out = run("this is not JSON and must not be echoed\n[]\n");
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.contains("must not be echoed"));
    assert!(text.contains("-32700"));
    assert!(text.contains("-32600"));
    let out = run(&format!("{}\n", "x".repeat(65_537)));
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("request_too_large")
    );
}
