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
    talk_under(&[], requests)
}

/// The same, with the server confined to the given directories.
fn talk_under(roots: &[&std::path::Path], requests: &[&str]) -> (Vec<Value>, String) {
    let mut lines = vec![
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    ];
    lines.extend_from_slice(requests);
    talk_raw(roots, &lines)
}

/// Sends exactly what it is given, with no handshake of its own -- for the
/// tests that are about the handshake, or about what the server does with
/// input no client would send.
fn talk_raw(roots: &[&std::path::Path], lines: &[&str]) -> (Vec<Value>, String) {
    let mut stdin = String::new();
    for l in lines {
        stdin.push_str(l);
        stdin.push('\n');
    }

    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("mcp");
    for root in roots {
        cmd.arg("--root").arg(root);
    }
    let out = cmd
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
fn test_the_server_offers_the_tools_it_has() {
    let (messages, _) = talk(&[r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#]);
    let mut tools = reply(&messages, 2)["result"]["tools"]
        .as_array()
        .expect("tools/list returns an array")
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    tools.sort();
    assert_eq!(
        tools,
        vec![
            "oinkie_compare".to_string(),
            "oinkie_extract".to_string(),
            "oinkie_info".to_string(),
            "oinkie_reaggregate".to_string(),
            "oinkie_run".to_string(),
        ]
    );
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

/// Produces a directory of element-wise similarity CSVs, the way `compare`
/// and `run` leave one, without needing a decompiler: the lifted files are
/// committed.
fn scored_directory(dir: &std::path::Path) -> std::path::PathBuf {
    let scores = dir.join("similarities");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", "op-set-jaccard", "-d"])
        .arg(&scores)
        .args([
            "testdata/hello_world/pcodes/hello_clang.json",
            "testdata/hello_world/pcodes/hello_gcc.json",
        ])
        .assert()
        .success();
    scores
}

fn call_tool(roots: &[&std::path::Path], name: &str, args: Value) -> Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": {"name": name, "arguments": args}
    })
    .to_string();
    let (messages, _) = talk_under(roots, &[&request]);
    reply(&messages, 2).clone()
}

fn call_reaggregate(root: &std::path::Path, args: Value) -> Value {
    call_tool(&[root], "oinkie_reaggregate", args)
}

/// The repository, canonicalized, since that is what a refusal compares
/// against and what a resolved path comes back as.
fn here() -> std::path::PathBuf {
    std::fs::canonicalize(".").unwrap()
}

/// The two lifted programs the parity tests compare.
///
/// Deliberately not the two hello worlds: those produce the same birthmark
/// under every analysis, so any pair of them scores 1.0 and a test built on
/// them passes for an implementation that returns 1.0 and nothing else.
const A: &str = "testdata/hello_world/pcodes/hello_clang.json";
const B: &str = "testdata/quoted_names/pcodes/udl.json";

fn similarities(result: &Value) -> Vec<f64> {
    result["result"]["structuredContent"]["scores"]
        .as_array()
        .unwrap_or_else(|| panic!("no scores in {result}"))
        .iter()
        .map(|s| s["similarity"].as_f64().unwrap())
        .collect()
}

/// The same analysis through the CLI, as the number it writes.
fn cli_run(dir: &std::path::Path, analysis: &str) -> Vec<f64> {
    let dest = dir.join("cli-run");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", analysis, "-s", "all", "-d"])
        .arg(&dest)
        .args([A, B])
        .assert()
        .success();
    std::fs::read_to_string(dest.join("results.csv"))
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("total duration"))
        .map(|l| l.split(',').nth(1).unwrap().parse::<f64>().unwrap())
        .collect()
}

/// The tool and the CLI have to agree, or the MCP surface is quietly its own
/// implementation of the same thing.
#[test]
fn test_reaggregating_gives_what_the_cli_gives() {
    let dir = tempfile::tempdir().unwrap();
    let scores = scored_directory(dir.path());

    let result = call_reaggregate(
        dir.path(),
        serde_json::json!({
            "score_directory": scores.to_str().unwrap(),
            "aggregator": "topn:1"
        }),
    );
    let served = result["result"]["structuredContent"]["scores"]
        .as_array()
        .expect("scores")
        .iter()
        .map(|s| s["similarity"].as_f64().unwrap())
        .collect::<Vec<_>>();
    assert!(!served.is_empty(), "{result}");

    // the same directory, the same aggregator, through the CLI
    let csv = dir.path().join("cli.csv");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["reaggregate", "-A", "topn:1", "-d"])
        .arg(&csv)
        .arg(&scores)
        .assert()
        .success();
    let mut from_cli = std::fs::read_to_string(&csv)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("total duration"))
        .map(|l| l.split(',').nth(1).unwrap().parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    let mut served_sorted = served.clone();
    from_cli.sort_by(f64::total_cmp);
    served_sorted.sort_by(f64::total_cmp);
    assert_eq!(served_sorted, from_cli);
}

/// The reason `--root` exists. The path came from a model, and this one is
/// somewhere the server was never pointed at.
#[test]
fn test_a_score_directory_outside_the_root_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let inside = dir.path().join("inside");
    std::fs::create_dir_all(&inside).unwrap();
    let outside = dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();

    let result = call_reaggregate(
        &inside,
        serde_json::json!({ "score_directory": outside.to_str().unwrap() }),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("outside every allowed"),
        "should have been refused: {result}"
    );
    // and the refusal says what may be used instead
    assert!(
        message.contains(inside.canonicalize().unwrap().to_str().unwrap()),
        "{message}"
    );
}

/// An aggregator the parser refuses is the caller's mistake, and has to come
/// back as one -- `Aggregator::from_str` fails with the library's catch-all,
/// which is deliberately not classified that way.
#[test]
fn test_an_unknown_aggregator_is_reported_as_the_callers_mistake() {
    let dir = tempfile::tempdir().unwrap();
    let scores = scored_directory(dir.path());
    let result = call_reaggregate(
        dir.path(),
        serde_json::json!({
            "score_directory": scores.to_str().unwrap(),
            "aggregator": "topn:N"
        }),
    );
    assert_eq!(
        result["error"]["code"].as_i64(),
        Some(-32602),
        "not reported as invalid params: {result}"
    );
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("oinkie_info"),
        "the refusal should say where the accepted names are: {result}"
    );
}

/// The point of the whole server: hand it two lifted programs and be told how
/// alike they are. Checked against the CLI, so the MCP surface cannot quietly
/// become a second implementation of the same computation.
#[test]
fn test_running_gives_what_the_cli_gives() {
    let dir = tempfile::tempdir().unwrap();
    let result = call_tool(
        &[&here()],
        "oinkie_run",
        serde_json::json!({"files": [A, B], "analysis": "op-set-jaccard", "strategy": "all"}),
    );
    let served = similarities(&result);

    assert_eq!(served.len(), 1, "one pair under 'all': {result}");
    assert!(
        served[0] > 0.0 && served[0] < 1.0,
        "these two must actually differ, or this test asserts nothing: {served:?}"
    );
    assert_eq!(served, cli_run(dir.path(), "op-set-jaccard"));
}

/// Extracting and then comparing has to reach the same number as running,
/// since it is the same computation with the birthmarks written down in
/// between.
#[test]
fn test_extract_then_compare_agrees_with_run() {
    let dir = tempfile::tempdir().unwrap();
    // one root for the inputs, one for what the tools write
    let birthmarks = dir.path().join("birthmarks");

    let extracted = call_tool(
        &[&here(), dir.path()],
        "oinkie_extract",
        serde_json::json!({
            "files": [A, B],
            "birthmark_type": "op-set",
            "dest": birthmarks.to_str().unwrap()
        }),
    );
    let written = extracted["result"]["structuredContent"]["birthmarks"]
        .as_array()
        .unwrap_or_else(|| panic!("no birthmarks in {extracted}"))
        .iter()
        .map(|b| b["output"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(written.len(), 2, "{extracted}");
    for w in &written {
        assert!(std::path::Path::new(w).exists(), "{w} was not written");
    }

    let compared = call_tool(
        &[&here(), dir.path()],
        "oinkie_compare",
        serde_json::json!({"files": written, "algorithm": "jaccard", "strategy": "all"}),
    );
    let two_step = similarities(&compared);

    let one_step = similarities(&call_tool(
        &[&here(), dir.path()],
        "oinkie_run",
        serde_json::json!({"files": [A, B], "analysis": "op-set-jaccard", "strategy": "all"}),
    ));
    assert_eq!(two_step, one_step);
    assert!(two_step[0] < 1.0, "{two_step:?}");
}

/// A directory `oinkie_run` wrote has to be one `oinkie_reaggregate` can read,
/// or the tools do not compose and the caller has to leave for the CLI.
#[test]
fn test_a_directory_run_wrote_can_be_reaggregated() {
    let dir = tempfile::tempdir().unwrap();
    let scores = dir.path().join("similarities");

    let run = call_tool(
        &[&here(), dir.path()],
        "oinkie_run",
        serde_json::json!({
            "files": [A, B],
            "analysis": "op-set-jaccard",
            "strategy": "all",
            "dest": scores.to_str().unwrap()
        }),
    );
    assert_eq!(
        run["result"]["structuredContent"]["dest"]
            .as_str()
            .map(std::path::Path::new),
        // canonicalized, which on macOS means /var became /private/var
        Some(std::fs::canonicalize(&scores).unwrap().as_path())
    );

    let again = call_reaggregate(
        dir.path(),
        serde_json::json!({"score_directory": scores.to_str().unwrap()}),
    );
    assert_eq!(similarities(&again), similarities(&run));
}

/// The library refuses a pairing whose algorithm does not operate on the
/// birthmark's shape, and names the one that was meant. That message is the
/// only place a model can learn the right spelling, so it has to survive.
#[test]
fn test_an_impossible_pairing_is_refused_in_the_librarys_words() {
    let result = call_tool(
        &[&here()],
        "oinkie_run",
        serde_json::json!({"files": [A], "analysis": "op-seq-euclidean"}),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("op-freq-euclidean"), "{result}");
    assert_eq!(result["error"]["code"].as_i64(), Some(-32602), "{result}");
}

/// A model handed a directory will pass all of it, and `all-and-self` is
/// quadratic. The count is known before anything is read, so the refusal
/// happens then.
#[test]
fn test_too_many_pairs_is_refused_before_anything_is_read() {
    let result = call_tool(
        &[&here()],
        "oinkie_run",
        serde_json::json!({"files": [A, B], "max_pairs": 1}),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("3 pairs"), "{result}");
    assert!(message.contains("max_pairs"), "{result}");
}

#[test]
fn test_a_program_outside_the_root_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let result = call_tool(
        &[dir.path()],
        "oinkie_run",
        serde_json::json!({"files": [std::fs::canonicalize(A).unwrap()]}),
    );
    assert!(
        result["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("outside every allowed"),
        "{result}"
    );
}

/// A client older than this build still gets a session. Every other test here
/// sends the version this was written against, so without this one the
/// negotiation is exercised at exactly one value -- the one that cannot fail.
#[test]
fn test_a_client_speaking_an_older_protocol_is_answered_in_its_own_version() {
    let (messages, _) = talk_raw(
        &[],
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"old","version":"0"}}}"#,
        ],
    );
    let result = &reply(&messages, 1)["result"];
    assert!(
        result["error"].is_null(),
        "an older client was refused: {result}"
    );
    assert_eq!(result["protocolVersion"].as_str(), Some("2024-11-05"));
}

/// A method the server does not have is a protocol error, not a crash and not
/// a silent nothing.
#[test]
fn test_an_unknown_method_is_refused_as_one() {
    let (messages, _) =
        talk(&[r#"{"jsonrpc":"2.0","id":9,"method":"tools/nonesuch","params":{}}"#]);
    // -32601 is JSON-RPC's "method not found"
    assert_eq!(reply(&messages, 9)["error"]["code"].as_i64(), Some(-32601));
}

/// Something that is not JSON at all arrives eventually -- a wrapper writing a
/// diagnostic into the pipe, a half-written line. The session has to survive
/// it, or one stray byte ends the server for good.
#[test]
fn test_a_line_that_is_not_json_does_not_end_the_session() {
    let (messages, _) = talk_raw(
        &[],
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "this is not json",
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/list","params":{}}"#,
        ],
    );
    let tools = reply(&messages, 5)["result"]["tools"]
        .as_array()
        .expect("the server answered after the garbage");
    assert_eq!(tools.len(), 5, "{messages:#?}");
}

/// A required argument left out is the caller's mistake and is reported inside
/// the result, as a tool error rather than a protocol one, so that a model
/// sees it and can try again. Worth pinning: it is rmcp's doing rather than
/// ours, and it is the only path where the two kinds are decided elsewhere.
#[test]
fn test_a_missing_required_argument_comes_back_as_a_tool_error() {
    let (messages, _) = talk(&[
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"oinkie_run","arguments":{}}}"#,
    ]);
    let result = &reply(&messages, 7)["result"];
    assert_eq!(result["isError"].as_bool(), Some(true), "{result}");
    let said = result["content"][0]["text"].as_str().unwrap_or_default();
    assert!(
        said.contains("files"),
        "the message has to name what is missing: {said}"
    );
}
