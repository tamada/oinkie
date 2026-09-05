//! The MCP server, driven the way a client drives it: JSON-RPC on stdin, JSON-RPC on stdout.
//!
//! End to end through the real binary rather than in-process, because the two
//! things most likely to go wrong are not reachable any other way. Whether
//! stdout carries anything but JSON-RPC is a property of the whole process --
//! a stray `println!` anywhere in the CLI would break a session and no
//! in-process test would see it. And whether the subcommand is wired up at all
//! is a question about `main`.
#![cfg(feature = "mcp")]

use assert_cmd::Command;
use oinkie::prelude::{AnalysisType, BirthmarkType};
use serde_json::Value;

/// One session: initialize, then whatever else is asked, then EOF -- which is
/// what stops the server.
fn talk(requests: &[&str]) -> (Vec<Value>, String) {
    let mut stdin = String::from(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
"#,
    );
    for r in requests {
        stdin.push_str(r);
        stdin.push('\n');
    }

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("mcp")
        .write_stdin(stdin)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8(out.stdout).expect("stdout is not UTF-8");
    let messages = stdout
        .lines()
        .map(|line| {
            serde_json::from_str(line).unwrap_or_else(|e| {
                panic!("stdout carried something that is not JSON: {line}: {e}")
            })
        })
        .collect();
    (messages, String::from_utf8(out.stderr).unwrap())
}

fn reply(messages: &[Value], id: i64) -> &Value {
    messages
        .iter()
        .find(|m| m["id"] == id)
        .unwrap_or_else(|| panic!("no reply to {id} in {messages:#?}"))
}

/// Over stdio, stdout *is* the JSON-RPC channel. One `println!` reaching it
/// corrupts the session, and the drivers in `cli/main.rs` print freely -- which
/// is why the tools call the library instead of going through them.
///
/// `talk` parses every line, so this test fails on any non-JSON output; the
/// assertions below only add that something was said at all.
#[test]
fn test_nothing_but_json_rpc_reaches_stdout() {
    let (messages, stderr) =
        talk(&[r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#]);
    assert!(messages.len() >= 2, "{messages:#?}");
    for m in &messages {
        assert_eq!(m["jsonrpc"], "2.0", "not a JSON-RPC message: {m}");
    }
    assert!(
        !stderr.contains("\"jsonrpc\""),
        "JSON-RPC leaked onto stderr: {stderr}"
    );
}

#[test]
fn test_the_server_offers_the_info_tool() {
    let (messages, _) = talk(&[r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#]);
    let tools = reply(&messages, 2)["result"]["tools"]
        .as_array()
        .expect("tools/list returns an array")
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(tools, vec!["oinkie_info".to_string()]);
}

/// The instructions are where a client is told that lifting happens elsewhere.
/// A model that is not told will ask for a tool that does not exist.
#[test]
fn test_initialize_says_lifting_is_not_offered_here() {
    let (messages, _) = talk(&[]);
    let instructions = reply(&messages, 1)["result"]["instructions"]
        .as_str()
        .expect("the server introduces itself");
    assert!(
        instructions.to_lowercase().contains("lift"),
        "{instructions}"
    );
}

/// What the tool serves is what the library generates. Asserted against the
/// library here, rather than against a list written in the test, so that the
/// test cannot drift with the code it is checking.
#[test]
fn test_the_vocabulary_served_is_the_one_the_library_generates() {
    let (messages, _) = talk(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"oinkie_info","arguments":{}}}"#,
    ]);
    let v = &reply(&messages, 2)["result"]["structuredContent"];

    let birthmarks = v["birthmarks"]
        .as_array()
        .expect("birthmarks")
        .iter()
        .map(|b| b["name"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        birthmarks,
        BirthmarkType::advertised()
            .map(|bt| bt.to_string())
            .collect::<Vec<_>>()
    );

    let analyses = v["analyses"]
        .as_array()
        .expect("analyses")
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        analyses,
        AnalysisType::advertised_names().collect::<Vec<_>>()
    );

    // and the result is structured rather than a wall of text to re-parse
    assert!(v["notes"].as_array().is_some_and(|n| !n.is_empty()));
}
