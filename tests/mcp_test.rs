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
use oinkie::birthmarks::BirthmarkType;
use oinkie::compare::Algorithm;
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
            "oinkie_review".to_string(),
            "oinkie_run".to_string(),
            "oinkie_stats".to_string(),
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
    // The server lists k-grams up to the ceiling `cli/vocabulary.rs` calls
    // `MAX_ADVERTISED_K`. It is written out here because that constant lives
    // in the binary, which a test cannot import -- and so that changing it
    // fails here, loudly, rather than passing by recomputing the same number.
    let ceiling = 8;
    assert_eq!(
        birthmarks,
        BirthmarkType::all(ceiling)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    let analyses = v["analyses"]
        .as_array()
        .expect("analyses")
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    // Every advertised birthmark crossed with the algorithms that operate on
    // its shape, in that order.
    let mut expected = Vec::new();
    for bt in BirthmarkType::all(ceiling) {
        for algorithm in Algorithm::ALL {
            if bt.pairs_with(algorithm) {
                expected.push(format!("{bt}-{}", algorithm.name()));
            }
        }
    }
    assert_eq!(analyses, expected);

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
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/hello_gcc.json",
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

fn call_review(root: &std::path::Path, args: Value) -> Value {
    call_tool(&[root], "oinkie_review", args)
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
const A: &str = "testdata/lifted/pcodes/hello_clang.json";
const B: &str = "testdata/lifted/pcodes/udl.json";

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
fn test_reviewing_gives_what_the_cli_gives() {
    let dir = tempfile::tempdir().unwrap();
    let scores = scored_directory(dir.path());

    let result = call_review(
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
        .args(["review", "-A", "topn:1", "-d"])
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

    let result = call_review(
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

/// A directory `oinkie_run` wrote, under `fc-set`, whose functions hold
/// 0, 0 and 3 elements (udl) and 1 (hello_clang).
fn fc_set_run(dir: &std::path::Path) -> std::path::PathBuf {
    let scores = dir.join("scores");
    let run = call_tool(
        &[&here(), dir],
        "oinkie_run",
        serde_json::json!({
            "files": [B, A],
            "analysis": "fc-set-jaccard",
            "strategy": "all",
            "dest": scores.to_str().unwrap()
        }),
    );
    assert!(run["error"].is_null(), "{run}");
    scores
}

/// `min_elements` is `--min-elements`: the same threshold, the same scores,
/// and the threshold said in the answer and in the CSV.
#[test]
fn test_review_drops_the_functions_with_too_few_elements() {
    let dir = tempfile::tempdir().unwrap();
    let scores = fc_set_run(dir.path());
    let out = dir.path().join("review.csv");
    let result = call_review(
        dir.path(),
        serde_json::json!({
            "score_directory": scores.to_str().unwrap(),
            "min_elements": "0.5x",
            "dest_file": out.to_str().unwrap()
        }),
    );
    assert!(result["error"].is_null(), "{result}");
    let s = similarities(&result);
    assert!((s[0] - 1.0 / 3.0).abs() < 1e-9, "{s:?}");
    let applied = &result["result"]["structuredContent"]["min_elements"];
    assert_eq!(applied["given"], "0.5x");
    assert_eq!(applied["threshold"], 0.5);
    let written = std::fs::read_to_string(&out).unwrap();
    assert_eq!(written.lines().last(), Some("min elements,0.5x,0.5"));
}

#[test]
fn test_a_bare_fraction_for_min_elements_is_the_callers_mistake() {
    let dir = tempfile::tempdir().unwrap();
    let scores = fc_set_run(dir.path());
    let result = call_review(
        dir.path(),
        serde_json::json!({
            "score_directory": scores.to_str().unwrap(),
            "min_elements": "0.3"
        }),
    );
    assert_eq!(result["error"]["code"].as_i64(), Some(-32602), "{result}");
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("0.3x"), "{message}");
}

/// The birthmarks are named by the score CSVs rather than by the caller, and
/// are read only under a root, like every other path.
#[test]
fn test_review_does_not_read_a_birthmark_outside_the_roots() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let scores = fc_set_run(dir.path());
    let pair = scores.join("00000.csv");
    let content = std::fs::read_to_string(&pair).unwrap();
    let left = content.lines().find(|l| l.starts_with("left,")).unwrap();
    let named = left.rsplit(',').next().unwrap();
    let moved = elsewhere.path().join("moved.json");
    std::fs::copy(named, &moved).unwrap();
    std::fs::write(&pair, content.replace(named, moved.to_str().unwrap())).unwrap();

    let result = call_review(
        dir.path(),
        serde_json::json!({
            "score_directory": scores.to_str().unwrap(),
            "min_elements": "2"
        }),
    );
    assert_eq!(result["error"]["code"].as_i64(), Some(-32602), "{result}");
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("outside every allowed"), "{message}");
}

/// A directory of op-seq birthmarks of A and B, and a lifted program beside
/// them that is not a birthmark and has to be reported as skipped.
fn birthmark_directory(dir: &std::path::Path) -> std::path::PathBuf {
    let birthmarks = dir.join("birthmarks");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["extract", "-b", "op-seq", "-d"])
        .arg(&birthmarks)
        .args([A, B])
        .assert()
        .success();
    std::fs::copy(A, birthmarks.join("not-a-birthmark.json")).unwrap();
    birthmarks
}

/// `oinkie_stats` is `oinkie stats -f json`: the same groups, top elements
/// included, and the same files skipped. Checked against the CLI so the two
/// cannot drift apart.
#[test]
fn test_stats_agrees_with_the_cli() {
    let dir = tempfile::tempdir().unwrap();
    let birthmarks = birthmark_directory(dir.path());
    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .args(["stats", "-f", "json", "--top", "2"])
        .arg(&birthmarks)
        .output()
        .unwrap();
    assert!(out.status.success());
    let cli: Value = serde_json::from_slice(&out.stdout).unwrap();

    let result = call_tool(
        &[dir.path()],
        "oinkie_stats",
        serde_json::json!({ "paths": [birthmarks.to_str().unwrap()], "top": 2 }),
    );
    assert!(result["error"].is_null(), "{result}");
    let served = &result["result"]["structuredContent"];
    assert_eq!(served["groups"], cli["groups"]);
    assert!(
        served["groups"][0]["top"]
            .as_array()
            .is_some_and(|t| t.len() == 2),
        "{served}"
    );
    let skipped = served["skipped"].as_array().unwrap();
    assert_eq!(skipped.len(), 1, "{served}");
    assert_eq!(skipped.len(), cli["skipped"].as_array().unwrap().len());
    // the per-file rows only when asked for
    assert!(served.get("files").is_none(), "{served}");
}

#[test]
fn test_stats_reports_each_file_when_asked() {
    let dir = tempfile::tempdir().unwrap();
    let birthmarks = birthmark_directory(dir.path());
    let result = call_tool(
        &[dir.path()],
        "oinkie_stats",
        serde_json::json!({ "paths": [birthmarks.to_str().unwrap()], "per_file": true }),
    );
    assert!(result["error"].is_null(), "{result}");
    let files = result["result"]["structuredContent"]["files"]
        .as_array()
        .unwrap_or_else(|| panic!("no per-file rows: {result}"));
    assert_eq!(files.len(), 2);
}

/// A directory contributes what is directly inside it, and what is below too
/// only with `recursive`.
#[test]
fn test_stats_descends_only_when_recursive() {
    let dir = tempfile::tempdir().unwrap();
    birthmark_directory(dir.path());
    for (recursive, groups) in [(false, 0), (true, 1)] {
        let result = call_tool(
            &[dir.path()],
            "oinkie_stats",
            serde_json::json!({ "paths": [dir.path().to_str().unwrap()], "recursive": recursive }),
        );
        assert!(result["error"].is_null(), "{result}");
        let found = result["result"]["structuredContent"]["groups"]
            .as_array()
            .unwrap()
            .len();
        assert_eq!(found, groups, "recursive: {recursive}");
    }
}

/// A directory inside the roots is confined, and so is every file found in
/// it: one can be a symlink out of them.
#[cfg(unix)]
#[test]
fn test_stats_does_not_read_through_a_symlink_out_of_the_roots() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let birthmarks = birthmark_directory(elsewhere.path());
    let inside = dir.path().join("inside");
    std::fs::create_dir_all(&inside).unwrap();
    let target = std::fs::read_dir(&birthmarks)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| !p.ends_with("not-a-birthmark.json"))
        .unwrap();
    std::os::unix::fs::symlink(&target, inside.join("linked.json")).unwrap();

    let result = call_tool(
        &[dir.path()],
        "oinkie_stats",
        serde_json::json!({ "paths": [inside.to_str().unwrap()] }),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("outside every allowed"),
        "should have been refused: {result}"
    );
}

/// An aggregator the parser refuses is the caller's mistake, and has to come
/// back as one -- `Aggregator::from_str` fails with the library's catch-all,
/// which is deliberately not classified that way.
#[test]
fn test_an_unknown_aggregator_is_reported_as_the_callers_mistake() {
    let dir = tempfile::tempdir().unwrap();
    let scores = scored_directory(dir.path());
    let result = call_review(
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
///
/// For every birthmark family and shape, not only `op-set`: a run that
/// compared the programs' operations whatever birthmark the analysis named
/// would agree on `op-set` alone.
#[test]
fn test_extract_then_compare_agrees_with_run() {
    for (birthmark_type, algorithm) in [
        ("op-set", "jaccard"),
        ("fc-set", "jaccard"),
        ("op-2gram-set", "jaccard"),
        ("fc-seq", "levenshtein"),
        ("op-3gram-seq", "levenshtein"),
        ("fc-freq", "cosine"),
    ] {
        let analysis = format!("{birthmark_type}-{algorithm}");
        let dir = tempfile::tempdir().unwrap();
        // one root for the inputs, one for what the tools write
        let birthmarks = dir.path().join("birthmarks");

        let extracted = call_tool(
            &[&here(), dir.path()],
            "oinkie_extract",
            serde_json::json!({
                "files": [A, B],
                "birthmark_type": birthmark_type,
                "dest": birthmarks.to_str().unwrap()
            }),
        );
        let written = extracted["result"]["structuredContent"]["birthmarks"]
            .as_array()
            .unwrap_or_else(|| panic!("{analysis}: no birthmarks in {extracted}"))
            .iter()
            .map(|b| b["output"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(written.len(), 2, "{analysis}: {extracted}");

        let compared = call_tool(
            &[&here(), dir.path()],
            "oinkie_compare",
            serde_json::json!({"files": written, "algorithm": algorithm, "strategy": "all"}),
        );
        let two_step = similarities(&compared);

        let one_step = similarities(&call_tool(
            &[&here(), dir.path()],
            "oinkie_run",
            serde_json::json!({"files": [A, B], "analysis": analysis, "strategy": "all"}),
        ));
        assert_eq!(two_step, one_step, "{analysis}");
        assert!(two_step[0] < 1.0, "{analysis}: {two_step:?}");
    }
}

/// Given somewhere to write, `oinkie_run` writes the birthmarks it compared
/// beside the scores, and each pair's CSV names them, as `run` does.
#[test]
fn test_run_writes_the_birthmarks_it_compares_when_given_a_dest() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("scores");
    let result = call_tool(
        &[&here(), dir.path()],
        "oinkie_run",
        serde_json::json!({
            "files": [A, B],
            "analysis": "fc-set-jaccard",
            "strategy": "all",
            "dest": dest.to_str().unwrap()
        }),
    );
    assert!(result["error"].is_null(), "{result}");
    let written = std::fs::read_dir(dest.join("birthmarks"))
        .unwrap_or_else(|e| panic!("no birthmarks directory: {e}"))
        .count();
    assert_eq!(written, 2);
    let pair = std::fs::read_to_string(dest.join("00000.csv")).unwrap();
    let left = pair
        .lines()
        .find(|l| l.starts_with("left,birthmark,"))
        .unwrap();
    let named = left.rsplit(',').next().unwrap();
    assert!(
        std::path::Path::new(named).exists(),
        "{named} does not exist"
    );
}

/// `dest` is confined, and so is what `oinkie_run` writes below it. A
/// `birthmarks` already there can be a symlink out of every root, and
/// resolving `dest` alone does not look through it.
#[cfg(unix)]
#[test]
fn test_run_does_not_write_birthmarks_through_a_symlink_out_of_the_roots() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let dest = dir.path().join("scores");
    std::fs::create_dir_all(&dest).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), dest.join("birthmarks")).unwrap();

    let result = call_tool(
        &[&here(), dir.path()],
        "oinkie_run",
        serde_json::json!({
            "files": [A, B],
            "strategy": "all",
            "dest": dest.to_str().unwrap()
        }),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("outside every allowed"),
        "should have been refused: {result}"
    );
    assert_eq!(
        std::fs::read_dir(elsewhere.path()).unwrap().count(),
        0,
        "something was written outside the roots"
    );
}

/// The same, one level down: the directory is real, and a birthmark's file
/// in it is the link -- to a file outside, or to nothing yet, which writing
/// would create.
#[cfg(unix)]
#[test]
fn test_run_does_not_write_a_birthmark_through_a_symlinked_file() {
    for target_exists in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let dest = dir.path().join("scores");
        // the name run gives A's birthmark, learned from a run into a scratch dest
        let scratch = dir.path().join("scratch");
        let first = call_tool(
            &[&here(), dir.path()],
            "oinkie_run",
            serde_json::json!({ "files": [A], "dest": scratch.to_str().unwrap() }),
        );
        assert!(first["error"].is_null(), "{first}");
        let name = std::fs::read_dir(scratch.join("birthmarks"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name();
        std::fs::create_dir_all(dest.join("birthmarks")).unwrap();
        let target = elsewhere.path().join("victim.json");
        if target_exists {
            std::fs::write(&target, "untouched").unwrap();
        }
        std::os::unix::fs::symlink(&target, dest.join("birthmarks").join(name)).unwrap();

        let result = call_tool(
            &[&here(), dir.path()],
            "oinkie_run",
            serde_json::json!({ "files": [A, B], "dest": dest.to_str().unwrap() }),
        );
        assert!(
            !result["error"].is_null(),
            "should have been refused (target exists: {target_exists}): {result}"
        );
        if target_exists {
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "untouched");
        } else {
            assert!(!target.exists(), "written through the link");
        }
    }
}

/// Plants `link` as a symlink to a file in `elsewhere`, which exists only when
/// `target_exists` -- a link to nothing is the other way through, since
/// writing to it creates the target. Returns the target.
#[cfg(unix)]
fn plant_link(
    link: &std::path::Path,
    elsewhere: &std::path::Path,
    target_exists: bool,
) -> std::path::PathBuf {
    let target = elsewhere.join("victim");
    if target_exists {
        std::fs::write(&target, "untouched").unwrap();
    }
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, link).unwrap();
    target
}

/// The call was refused, and nothing reached the link's target.
#[cfg(unix)]
fn assert_refused_and_untouched(result: &Value, target: &std::path::Path, target_exists: bool) {
    let message = result["error"]["message"].as_str().unwrap_or_default();
    // refused by the confinement, not by something else going wrong
    let expected = if target_exists {
        "outside every allowed"
    } else {
        "symlink"
    };
    assert!(
        message.contains(expected),
        "should have been refused (target exists: {target_exists}): {result}"
    );
    if target_exists {
        assert_eq!(std::fs::read_to_string(target).unwrap(), "untouched");
    } else {
        assert!(!target.exists(), "written through the link");
    }
}

/// The pairs' CSVs and their index are files `oinkie_run` and
/// `oinkie_compare` name below `dest`, and an existing one can be a link out
/// of every root, which resolving `dest` does not look through.
#[cfg(unix)]
#[test]
fn test_run_and_compare_do_not_write_scores_through_a_symlinked_file() {
    let dir = tempfile::tempdir().unwrap();
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
        .map(|b| b["output"].clone())
        .collect::<Vec<_>>();

    for (n, (tool, files)) in [
        ("oinkie_run", serde_json::json!([A, B])),
        ("oinkie_compare", Value::Array(written)),
    ]
    .into_iter()
    .enumerate()
    {
        for (m, name) in ["00000.csv", "results.csv"].into_iter().enumerate() {
            for target_exists in [true, false] {
                let elsewhere = tempfile::tempdir().unwrap();
                let dest = dir.path().join(format!("scores{n}{m}{target_exists}"));
                let target = plant_link(&dest.join(name), elsewhere.path(), target_exists);
                let result = call_tool(
                    &[&here(), dir.path()],
                    tool,
                    serde_json::json!({
                        "files": files,
                        "strategy": "all",
                        "dest": dest.to_str().unwrap()
                    }),
                );
                assert_refused_and_untouched(&result, &target, target_exists);
            }
        }
    }
}

/// `oinkie_extract` names its birthmark files itself, below the `dest` it
/// was given, and one already there can be a link out of every root.
#[cfg(unix)]
#[test]
fn test_extract_does_not_write_a_birthmark_through_a_symlinked_file() {
    for target_exists in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        // the name extract gives A's birthmark, learned from a scratch dest
        let scratch = dir.path().join("scratch");
        let first = call_tool(
            &[&here(), dir.path()],
            "oinkie_extract",
            serde_json::json!({ "files": [A], "dest": scratch.to_str().unwrap() }),
        );
        assert!(first["error"].is_null(), "{first}");
        let name = std::fs::read_dir(&scratch)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name();
        let dest = dir.path().join("birthmarks");
        let target = plant_link(&dest.join(name), elsewhere.path(), target_exists);

        let result = call_tool(
            &[&here(), dir.path()],
            "oinkie_extract",
            serde_json::json!({ "files": [A, B], "dest": dest.to_str().unwrap() }),
        );
        assert_refused_and_untouched(&result, &target, target_exists);
    }
}

/// `oinkie_review`'s `dest_file` is named by the caller and resolved as given,
/// which looks through a link at its end. Pinned with the others so that it
/// stays that way.
#[cfg(unix)]
#[test]
fn test_review_does_not_write_through_a_symlinked_dest_file() {
    for target_exists in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let scores = scored_directory(dir.path());
        let dest_file = dir.path().join("reviewed.csv");
        let target = plant_link(&dest_file, elsewhere.path(), target_exists);
        let result = call_review(
            dir.path(),
            serde_json::json!({
                "score_directory": scores.to_str().unwrap(),
                "dest_file": dest_file.to_str().unwrap()
            }),
        );
        assert_refused_and_untouched(&result, &target, target_exists);
    }
}

/// `oinkie_extract` given one file twice, or two copies of one program under
/// one stem, extracts each birthmark file once rather than having two workers
/// write it at the same time. Counted from what it reports, since the race
/// seldom loses and the files alone would pass without the fix.
#[test]
fn test_extract_writes_each_birthmark_file_once() {
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("copy/hello_clang.json");
    std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
    std::fs::copy(A, &copy).unwrap();
    let dest = dir.path().join("birthmarks");
    let result = call_tool(
        &[&here(), dir.path()],
        "oinkie_extract",
        serde_json::json!({
            "files": [A, B, A, copy.to_str().unwrap()],
            "dest": dest.to_str().unwrap()
        }),
    );
    let reported = result["result"]["structuredContent"]["birthmarks"]
        .as_array()
        .unwrap_or_else(|| panic!("no birthmarks in {result}"));
    assert_eq!(reported.len(), 2, "{result}");
    let written = std::fs::read_dir(&dest)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(written.len(), 2, "{written:?}");
    for w in written {
        let text = std::fs::read_to_string(&w).unwrap();
        serde_json::from_str::<Value>(&text).unwrap_or_else(|e| panic!("{}: {e}", w.display()));
    }
}

/// The same input twice is one birthmark, written once, rather than two
/// workers writing one file at the same time.
#[test]
fn test_run_given_an_input_twice_writes_its_birthmark_once() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("scores");
    let result = call_tool(
        &[&here(), dir.path()],
        "oinkie_run",
        serde_json::json!({
            "files": [A, A, B],
            "strategy": "all",
            "dest": dest.to_str().unwrap()
        }),
    );
    assert!(result["error"].is_null(), "{result}");
    let written = std::fs::read_dir(dest.join("birthmarks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(written.len(), 2, "{written:?}");
    for w in written {
        let text = std::fs::read_to_string(&w).unwrap();
        serde_json::from_str::<serde_json::Value>(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", w.display()));
    }
}

/// A directory `oinkie_run` wrote has to be one `oinkie_review` can read,
/// or the tools do not compose and the caller has to leave for the CLI.
#[test]
fn test_a_directory_run_wrote_can_be_reviewed() {
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

    let again = call_review(
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

/// A k-gram of size zero is refused as the caller's mistake, before any
/// extraction -- which would otherwise fail the tool's worker on a window of
/// size zero.
#[test]
fn test_a_zero_gram_analysis_is_refused_as_the_callers_mistake() {
    let result = call_tool(
        &[&here()],
        "oinkie_run",
        serde_json::json!({"files": [A, B], "analysis": "op-0gram-set-jaccard", "strategy": "all"}),
    );
    let message = result["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("op-0gram-set"), "{result}");
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
    let message = reply(&messages, 1);
    // The top-level `error`, not `result.error`. The latter is null whether
    // the call succeeded or failed -- on failure there is no `result` at all,
    // and indexing null yields null -- so asserting on it could not fail.
    assert!(
        message["error"].is_null(),
        "an older client was refused: {message}"
    );
    assert_eq!(
        message["result"]["protocolVersion"].as_str(),
        Some("2024-11-05")
    );
}

/// A version the server has never heard of is answered in one it does support,
/// rather than refused. Worth pinning because it is not the obvious behaviour:
/// a client from the future, or one with a typo, still gets a usable session
/// instead of an error it would have to interpret.
#[test]
fn test_a_version_the_server_does_not_know_falls_back_rather_than_failing() {
    let (messages, _) = talk_raw(
        &[],
        &[
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01","capabilities":{},"clientInfo":{"name":"odd","version":"0"}}}"#,
        ],
    );
    let message = reply(&messages, 1);
    assert!(message["error"].is_null(), "{message}");
    let negotiated = message["result"]["protocolVersion"]
        .as_str()
        .expect("a version was still agreed");
    assert_ne!(negotiated, "1999-01-01", "that version does not exist");
    assert!(
        negotiated.starts_with("202"),
        "and the fallback should be a real one: {negotiated}"
    );
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
    assert_eq!(tools.len(), 6, "{messages:#?}");
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
