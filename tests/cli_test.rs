use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

// Tests marked `#[serial(ghidra)]` start a real Ghidra. They are serialised
// against each other because Ghidra compiles its SLEIGH language definitions
// on first use and caches them *inside its own installation*, so two headless
// runs against an installation nobody has used yet write the same file at the
// same time and the loser reads a half-written one. analyzeHeadless exits
// successfully regardless, so it arrives as a missing output rather than an
// error (#54).
//
// The group is named for what is shared rather than left anonymous, so that a
// future test needing the same exclusion knows which one to join.

#[test]
fn test_info_command() {
    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Oinkie Info"))
        .stdout(predicate::str::contains("Birthmarks"))
        .stdout(predicate::str::contains("Compare Algorithms"));
}

#[test]
#[serial(ghidra)]
fn test_lift_command() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    let result = cmd
        .arg("lift")
        .arg("-d")
        .arg(&dest)
        .arg("-r")
        .arg("ghidra-pcode")
        .arg("testdata/bin/hello_clang")
        .assert();

    result.success();
    let out_file = dest.join("hello_clang.json");
    assert!(out_file.exists(), "hello_clang.json was not generated");
}

/// Nothing inside the library can check that the environment variable is
/// actually read: the search takes its environment as a parameter now, so
/// that the tests stop writing to the process' own (#24), and a test that
/// injects the lookup cannot also prove the real one is wired to it.
///
/// A child process can. `GHIDRA_HOME` is set for that process alone, which is
/// hermetic in the way `set_var` never was, and pointing it at a directory
/// with no `support/analyzeHeadless` makes the run fail while naming the path
/// it was given -- so the assertion is that the value reached Ghidra's home,
/// not merely that the run failed. No Ghidra starts, so this does not join
/// the `ghidra` group.
///
/// The home is a fixed absolute path rather than one under the temporary
/// directory. It only has to be somewhere Ghidra is not, and naming it
/// literally keeps the expected string a literal too. Derived from `TMPDIR`,
/// it would have to survive both `to_str` and the `{:?}` the error message
/// formats it with -- a non-UTF-8 `TMPDIR` panics on the first, and one
/// holding a quote or a backslash comes back escaped from the second. Either
/// way the test would report the environment variable as broken on a machine
/// where the only unusual thing is where it puts its temporary files.
///
/// The two halves are asserted separately rather than as the joined path,
/// because both the separator and the entry point's name belong to the
/// platform: `join` writes a backslash on Windows, `{:?}` then escapes it, and
/// the entry point there is `analyzeHeadless.bat` (#136). What the test is
/// about is that the value arrived -- the home in the message -- and that
/// Ghidra is what was looked for, and neither of those is a question about
/// path syntax.
#[test]
fn test_the_lifter_home_is_read_from_the_environment() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    Command::cargo_bin("oinkie")
        .unwrap()
        .env("GHIDRA_HOME", "/oinkie-no-such-ghidra")
        .arg("lift")
        .arg("-d")
        .arg(&dest)
        .arg("testdata/bin/hello_clang")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("oinkie-no-such-ghidra")
                .and(predicate::str::contains("analyzeHeadless")),
        );
}

/// clap's own message begins "error: ", and `main` used to print it behind
/// "Error: " -- so a mistyped flag came back as "Error: error: ..." (#62).
/// oinkie's own errors carry no prefix of their own and keep theirs.
///
/// End to end because the doubling was in `main`, which no unit test reaches:
/// fixing only the `Display` arm of `Error::Clap` would have changed nothing
/// a user sees.
#[test]
fn test_a_usage_error_is_not_prefixed_twice() {
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("run")
        .arg("--bogus-flag")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("error: "))
        .stderr(predicate::str::contains("Error: error:").not());

    // and the other arm still says whose error it is
    let temp_dir = tempdir().unwrap();
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(temp_dir.path().join("birthmarks"))
        .arg("no-such-file.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: IO error for"));
}

#[test]
fn test_extract_command() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("birthmarks");

    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("extract")
        .arg("-d")
        .arg(&dest)
        .arg("-b")
        .arg("op-seq")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .arg("testdata/lifted/pcodes/hello_gcc.json")
        .assert()
        .success();

    // Verify the output files were created
    let entries: Vec<_> = fs::read_dir(&dest)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "Expected 2 birthmark files to be generated"
    );
}

/// `-b` names a birthmark the way the library does now. It used to be a
/// `ValueEnum` whose clap names came from Rust identifiers, so a 3-gram was
/// `op-tri-gram-set` on the command line, `op-3gram-set` everywhere else, and
/// neither the docs' spelling nor the library's parsed (#25).
///
/// End to end rather than at the parser, because this is the half a user
/// types: the new spelling has to reach an extracted file, and the old one
/// has to fail rather than silently mean something else.
#[test]
fn test_extract_names_a_kgram_the_way_the_library_does() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("birthmarks");

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(&dest)
        .arg("-b")
        .arg("op-3gram-set")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .assert()
        .success();
    assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(temp_dir.path().join("unused"))
        .arg("-b")
        .arg("op-tri-gram-set")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown birthmark type"));
}

/// `op-*gram-freq` birthmarks could not be written at all: a k-gram is a list
/// of operations, a JSON object's keys are strings, and `serde_json` refused
/// the map outright (#59). Eight of the thirty birthmark types the CLI
/// advertises — `op-1gram-freq` through `op-8gram-freq` — could not produce a
/// file, and with them twenty-four of its eighty analyses.
///
/// `op-2gram-freq` on this fixture rather than a larger k, because the bug
/// hid behind emptiness: the fixture's one function has four operations, so
/// k >= 5 yields an empty map and an empty map has no key to refuse. A test
/// that happened to pick k = 5 would have passed against the bug.
///
/// The score is checked against `run`, which computes the same analysis
/// without ever writing a birthmark. Equal scores say the file round trip is
/// faithful, not merely that it completed.
#[test]
fn test_a_kgram_frequency_birthmark_can_be_written_and_read_back() {
    let temp_dir = tempdir().unwrap();
    let birthmarks = temp_dir.path().join("birthmarks");
    let through_a_file = temp_dir.path().join("through-a-file");
    let in_memory = temp_dir.path().join("in-memory");
    let inputs = [
        "testdata/lifted/pcodes/hello_clang.json",
        "testdata/lifted/pcodes/hello_gcc.json",
    ];

    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["extract", "-b", "op-2gram-freq", "-d"])
        .arg(&birthmarks)
        .args(inputs)
        .assert()
        .success();
    let written: Vec<_> = fs::read_dir(&birthmarks)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(written.len(), 2, "the birthmarks were not written");
    assert!(
        fs::read_to_string(&written[0])
            .unwrap()
            .contains("KgramFreq"),
        "the file does not hold what was asked for"
    );

    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["compare", "-a", "cosine", "-d"])
        .arg(&through_a_file)
        .args(&written)
        .assert()
        .success();

    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", "op-2gram-freq-cosine", "-d"])
        .arg(&in_memory)
        .args(inputs)
        .assert()
        .success();

    // Every row, sorted. The rows are written from a parallel iteration, so
    // which one lands first is not part of the output's meaning -- reading
    // only the first line made this test depend on a race, and it lost.
    //
    // Each row is `index, similarity, left, right, duration`; the duration is
    // wall clock and the only field that differs between two runs of the same
    // analysis, so it is dropped.
    let scores = |dir: &std::path::Path| {
        let csv = fs::read_to_string(dir.join("results.csv")).unwrap();
        let mut rows: Vec<String> = csv
            .lines()
            .filter(|l| !l.starts_with("total duration,"))
            .map(|l| l.rsplit_once(',').unwrap().0.to_string())
            .collect();
        rows.sort();
        assert!(!rows.is_empty(), "no scores in {}", dir.display());
        rows
    };
    assert_eq!(scores(&through_a_file), scores(&in_memory));
}

#[test]
fn test_compare_command() {
    let temp_dir = tempdir().unwrap();
    let birthmarks_dir = temp_dir.path().join("birthmarks");
    let similarities_dir = temp_dir.path().join("similarities");

    // First, extract
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(&birthmarks_dir)
        .arg("-b")
        .arg("op-seq")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .arg("testdata/lifted/pcodes/hello_gcc.json")
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&birthmarks_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    // Now, compare
    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("compare")
        .arg("-d")
        .arg(&similarities_dir)
        .arg("-a")
        .arg("jaccard")
        .arg("-A")
        .arg("hungarian")
        .arg("-s")
        .arg("all")
        .args(&entries)
        .assert()
        .success();

    let sim_entries: Vec<_> = fs::read_dir(&similarities_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(
        !sim_entries.is_empty(),
        "Expected similarity files to be generated"
    );
}

#[test]
fn test_run_command() {
    let temp_dir = tempdir().unwrap();
    let similarities_dir = temp_dir.path().join("similarities");

    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("run")
        .arg("-a")
        .arg("op-set-jaccard")
        .arg("-s")
        .arg("all")
        .arg("-d")
        .arg(&similarities_dir)
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .arg("testdata/lifted/pcodes/hello_gcc.json")
        .assert()
        .success();

    let sim_entries: Vec<_> = fs::read_dir(&similarities_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(
        !sim_entries.is_empty(),
        "Expected similarity files to be generated"
    );
}

/// The first score in a summary CSV: `index,similarity,left,right,duration`.
fn first_score(summary: &std::path::Path) -> f64 {
    let content =
        fs::read_to_string(summary).unwrap_or_else(|e| panic!("{}: {e}", summary.display()));
    let first = content.lines().next().expect("an empty summary");
    first.split(',').nth(1).unwrap().parse().unwrap()
}

/// `run` is `extract` followed by `compare`, so the two give the same score --
/// for every birthmark family and shape. `run` once compared the programs'
/// operations whatever birthmark the analysis named, so every `fc-*` and
/// k-gram analysis reported an `op-*` score under the wrong name (#150).
#[test]
fn test_run_scores_what_extract_and_compare_score() {
    let inputs = [
        "testdata/lifted/pcodes/hello_clang.json",
        "testdata/lifted/pcodes/udl.json",
    ];
    for (birthmark_type, algorithm) in [
        ("op-seq", "levenshtein"),
        ("fc-seq", "levenshtein"),
        ("op-3gram-seq", "levenshtein"),
        ("fc-set", "jaccard"),
        ("op-2gram-set", "jaccard"),
        ("fc-freq", "cosine"),
        ("op-2gram-freq", "weighted-jaccard"),
    ] {
        let analysis = format!("{birthmark_type}-{algorithm}");
        let dir = tempdir().unwrap();

        Command::cargo_bin("oinkie")
            .unwrap()
            .args(["run", "-a", &analysis, "-s", "all", "-d"])
            .arg(dir.path().join("run"))
            .args(inputs)
            .assert()
            .success();

        let birthmarks = dir.path().join("birthmarks");
        Command::cargo_bin("oinkie")
            .unwrap()
            .args(["extract", "-b", birthmark_type, "-d"])
            .arg(&birthmarks)
            .args(inputs)
            .assert()
            .success();
        let mut extracted = fs::read_dir(&birthmarks)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();
        // `compare` pairs its arguments in order; give them in the order `run` had
        extracted.sort_by_key(|p| {
            !p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("hello_clang")
        });
        Command::cargo_bin("oinkie")
            .unwrap()
            .args(["compare", "-a", algorithm, "-s", "all", "-d"])
            .arg(dir.path().join("compare"))
            .args(&extracted)
            .assert()
            .success();

        let run = first_score(&dir.path().join("run").join("results.csv"));
        let compare = first_score(&dir.path().join("compare").join("results.csv"));
        assert_eq!(
            run, compare,
            "{analysis}: run and extract + compare disagree"
        );
    }
}

/// A k-gram of size zero is a usage error, not a panic -- `run` reached
/// extraction with it once `run` extracted at all (#150).
#[test]
fn test_a_zero_gram_analysis_is_refused_not_a_panic() {
    let dir = tempdir().unwrap();
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", "op-0gram-set-jaccard", "-s", "all", "-d"])
        .arg(dir.path())
        .args([
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/udl.json",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("op-0gram-set"))
        .stderr(predicates::str::contains("panicked").not());
}

/// The birthmark named on a side of a pair's CSV: the last field of its
/// `left` or `right` line.
fn side_birthmark(pair_csv: &std::path::Path, side: &str) -> std::path::PathBuf {
    let content = fs::read_to_string(pair_csv).unwrap();
    let line = content
        .lines()
        .find(|l| l.starts_with(&format!("{side},birthmark,")))
        .unwrap_or_else(|| panic!("no {side} birthmark in {}", pair_csv.display()));
    std::path::PathBuf::from(line.rsplit(',').next().unwrap())
}

/// `run` writes the birthmarks it compares into the score directory, one per
/// input, and each pair's CSV names the two it used -- so the directory holds
/// everything `review` needs to re-read the comparison, as `compare`'s does
/// (#128).
#[test]
fn test_run_writes_the_birthmarks_it_compares() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("out");
    let inputs = [
        "testdata/lifted/pcodes/hello_clang.json",
        "testdata/lifted/pcodes/hello_gcc.json",
        "testdata/lifted/pcodes/udl.json",
    ];
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", "fc-set-jaccard", "-s", "all", "-d"])
        .arg(&dest)
        .args(inputs)
        .assert()
        .success();

    let written = fs::read_dir(dest.join("birthmarks"))
        .expect("no birthmarks directory")
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(
        written.len(),
        inputs.len(),
        "one birthmark per input: {written:?}"
    );

    for pair in ["00000.csv", "00001.csv", "00002.csv"] {
        for side in ["left", "right"] {
            let named = side_birthmark(&dest.join(pair), side);
            assert!(
                written.contains(&named),
                "{pair} {side}: {} was not written",
                named.display()
            );
            let b: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&named).unwrap()).unwrap();
            assert_eq!(
                b["metadata"]["birthmark_type"],
                "FcSet",
                "{}",
                named.display()
            );
        }
    }
}

/// `--skip` reuses a birthmark already written -- but only one of the type the
/// analysis asks for. The file name does not say which type a birthmark is, so
/// a directory run before with another analysis would otherwise hand the old
/// birthmarks to the new comparison.
#[test]
fn test_run_skip_does_not_reuse_a_birthmark_of_another_type() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("out");
    let inputs = [
        "testdata/lifted/pcodes/hello_clang.json",
        "testdata/lifted/pcodes/udl.json",
    ];
    let run = |analysis: &str, scores: &str| {
        Command::cargo_bin("oinkie")
            .unwrap()
            .args(["run", "-S", "-a", analysis, "-s", "all", "-d"])
            .arg(dest.join(scores))
            .args(inputs)
            .assert()
            .success();
    };
    run("fc-set-jaccard", "");
    run("op-set-jaccard", "");
    let named = side_birthmark(&dest.join("00000.csv"), "left");
    let b: serde_json::Value = serde_json::from_str(&fs::read_to_string(&named).unwrap()).unwrap();
    assert_eq!(
        b["metadata"]["birthmark_type"], "OpSet",
        "the fc-set birthmark was reused"
    );
    // and the score was recomputed with it: one kept from the fc-set run
    // would name the op-set birthmark that replaced the one it compared
    let fresh = dir.path().join("fresh");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-a", "op-set-jaccard", "-s", "all", "-d"])
        .arg(&fresh)
        .args(inputs)
        .assert()
        .success();
    let similarity = |pair: &std::path::Path| -> String {
        let content = fs::read_to_string(pair).unwrap();
        let line = content.lines().find(|l| l.starts_with("result,")).unwrap();
        line.split(',').nth(2).unwrap().to_string()
    };
    assert_eq!(
        similarity(&dest.join("00000.csv")),
        similarity(&fresh.join("00000.csv")),
        "the fc-set score was kept"
    );
}

/// The same input twice is extracted once and written once, rather than by
/// two workers into one file at the same time.
#[test]
fn test_run_given_an_input_twice_writes_its_birthmark_once() {
    let dir = tempdir().unwrap();
    let dest = dir.path().join("out");
    Command::cargo_bin("oinkie")
        .unwrap()
        .args(["run", "-s", "all", "-d"])
        .arg(&dest)
        .args([
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/hello_clang.json",
            "testdata/lifted/pcodes/udl.json",
        ])
        .assert()
        .success();
    let written = fs::read_dir(dest.join("birthmarks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(written.len(), 2, "{written:?}");
    for w in written {
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&w).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", w.display()));
    }
}

/// The old name is refused, not aliased, and the refusal names the new one --
/// including when it is handed the arguments the old command took.
#[test]
fn test_the_old_name_reaggregate_is_refused_naming_review() {
    for args in [
        vec!["reaggregate"],
        vec!["reaggregate", "-A", "topn:3", "-d", "x.csv", "out/"],
    ] {
        Command::cargo_bin("oinkie")
            .unwrap()
            .args(&args)
            .assert()
            .failure()
            .stderr(predicates::str::contains("use review"));
    }
}

#[test]
fn test_review_command() {
    let temp_dir = tempdir().unwrap();
    let birthmarks_dir = temp_dir.path().join("birthmarks");
    let similarities_dir = temp_dir.path().join("similarities");

    // First, extract
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(&birthmarks_dir)
        .arg("-b")
        .arg("op-seq")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .arg("testdata/lifted/pcodes/hello_gcc.json")
        .assert()
        .success();

    let entries: Vec<_> = fs::read_dir(&birthmarks_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    // Compare
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("compare")
        .arg("-d")
        .arg(&similarities_dir)
        .arg("-a")
        .arg("levenshtein")
        .arg("-s")
        .arg("all")
        .args(&entries)
        .assert()
        .success();

    // Review
    let dest_file = temp_dir.path().join("review.csv");
    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("review")
        .arg("-A")
        .arg("hungarian")
        .arg("-d")
        .arg(&dest_file)
        .arg(&similarities_dir)
        .assert()
        .success();

    assert!(dest_file.exists());
}

/// The path given to `-i` is handed to Ghidra as its project location and is
/// also used as the working directory of the Ghidra process. A relative path
/// must therefore be resolved before the process starts, or Ghidra resolves it
/// a second time against itself and looks for `irs/irs`.
/// Both `-i` regressions in one run, because each run is a whole decompiler.
///
/// A path given to `-i` is created rather than required to exist, as every
/// other destination directory in the CLI is and as the temporary directory
/// used without `-i` is by construction. And it is resolved once: it used to
/// be resolved twice, by us and again by Ghidra against its own working
/// directory, so `-i irs` went looking for `irs/irs`.
///
/// A relative path that does not exist yet covers both at once, and being
/// nested covers creating intermediate levels rather than just the last.
///
/// Either regression makes the run itself fail here, which is how #36 was
/// reported -- Ghidra complaining that `irs/irs` did not exist -- rather than
/// showing up as a stray directory. Both were checked by reverting each fix in
/// turn. The explicit assertions below still earn their place: they name which
/// of the two broke, and they catch a variant that doubles the path without
/// failing outright.
#[test]
#[serial(ghidra)]
fn test_lift_command_intermediate_dir_is_created_and_resolved_once() {
    // Ghidra rejects any path element starting with '.', and tempdir() names
    // its directories ".tmpXXXX", so the project location needs a plain prefix.
    let temp_dir = tempfile::Builder::new()
        .prefix("oinkie_test")
        .tempdir()
        .unwrap();
    let input = fs::canonicalize("testdata/bin/hello_clang").unwrap();
    let dest = temp_dir.path().join("lifted");

    Command::cargo_bin("oinkie")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg("lift")
        .arg("-i")
        .arg("irs/nested")
        .arg("-d")
        .arg(&dest)
        .arg(&input)
        .assert()
        .success();

    let intermediate = temp_dir.path().join("irs/nested");
    assert!(intermediate.is_dir(), "the -i directory was not created");
    assert!(
        !intermediate.join("irs/nested").exists(),
        "the -i path was resolved twice"
    );
    assert!(dest.join("hello_clang.json").exists());
}

/// A function name can hold a double quote. A C++ user-defined literal
/// operator is the everyday way to get one: Ghidra demangles `operator""_km`
/// as `operator""__km`, and the lifting script used to paste names into the
/// JSON unescaped, so the file it produced could not be read back.
///
/// `lift` reported success either way -- the script wrote its bytes and
/// returned, and analyzeHeadless exits 0 regardless (#54) -- so the failure
/// only appeared later, at `extract` (#77).
///
/// Both halves are asserted. That the output parses is the bug; that the name
/// survives is what stops the fix from being "strip the quote", which would
/// parse and would then compare a name the program does not have.
#[test]
#[serial(ghidra)]
fn test_lift_escapes_a_quote_in_a_function_name() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("lift")
        .arg("-d")
        .arg(&dest)
        .arg("testdata/bin/udl")
        .assert()
        .success();

    let out_file = dest.join("udl.json");
    // through oinkie's own reader rather than a parser of the test's
    // choosing: this is the operation that used to fail
    oinkie::Program::load(&out_file).expect("oinkie cannot read the file it just wrote");

    let body = fs::read_to_string(&out_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let names = json["functions"]
        .as_array()
        .expect("no functions array")
        .iter()
        .map(|f| f["name"].as_str().expect("a function with no name"))
        .collect::<Vec<_>>();
    assert!(
        names.iter().any(|n| n.contains('"')),
        "the quoted name did not survive escaping: {names:?}"
    );
}

/// A lifting script can write bytes, return normally, and leave behind a file
/// nothing can read. `analyzeHeadless` exits 0 either way, so before #83 that
/// was a successful lift, and the failure surfaced at `extract` as a parse
/// error naming a line and column in a file the reader had never seen.
///
/// Driven through `--script`, which is the path that cannot be fixed by
/// correcting the built-in script: a replacement is arbitrary Java that oinkie
/// never inspects, so the check has to be on the output.
#[test]
#[serial(ghidra)]
fn test_a_lift_whose_output_cannot_be_read_is_not_a_successful_lift() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("lift")
        .arg("--script")
        .arg("testdata/scripts/BrokenLifter.java")
        .arg("-d")
        .arg(&dest)
        .arg("testdata/bin/hello_clang")
        .assert()
        .failure()
        // The binary the user asked about, matched as the whole path and with
        // the colon that follows it. Matching "hello_clang" alone proved
        // nothing: the output is named hello_clang.json, so the assertion
        // passed on a message that did not mention the binary at all.
        .stderr(predicate::str::contains("testdata/bin/hello_clang:"))
        // the file that is wrong, which the cause names
        .stderr(predicate::str::contains("hello_clang.json"))
        // and that the lifter claimed success, which is what points at the script
        .stderr(predicate::str::contains("reported success"));
}

/// Birthmarks for the `stats` tests below, extracted rather than committed:
/// `testdata/` holds lifted programs and no birthmark, and the extraction is
/// a second of work that keeps the fixtures one thing rather than two that
/// can disagree.
///
/// The names carry a content hash -- `hello_clang_48fa2ffd841490f0.json` --
/// so the assertions below use the `file_name` column, which is the stem.
fn birthmarks_in(dir: &std::path::Path) {
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(dir)
        .arg("-b")
        .arg("op-seq")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .arg("testdata/lifted/pcodes/hello_gcc.json")
        .assert()
        .success();
}

/// `--per-file` with `-f csv` is a whole output function of its own, and
/// nothing ran it: `cli/stats.rs` has unit tests for the arithmetic and none
/// of the tests started the command, so the formatters and the driver that
/// picks between them were never executed. A renamed column or a row built in
/// the wrong order would have shipped.
///
/// The header is asserted in full rather than by one column, because what
/// breaks silently here is a swap: every name still present, in the wrong
/// place, over data that still parses as CSV.
#[test]
fn test_stats_writes_a_row_per_file_as_csv() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("csv")
        .arg("--per-file")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(
        lines[0],
        "path,file_name,ir,birthmark_type,functions,empty,\
elements_min,elements_q1,elements_median,elements_q3,elements_max,\
elements_mean,elements_stddev"
    );
    assert_eq!(lines.len(), 3, "expected a header and two rows: {text}");
    for name in ["hello_clang", "hello_gcc"] {
        assert!(
            lines[1..].iter().any(|l| l.contains(&format!(",{name},"))),
            "no row for {name}: {text}"
        );
    }
}

/// The same for Markdown, which reaches a different function: the per-file
/// section is built where the summary tables are, and only when `--per-file`
/// is given, so the two formats share no code on this path.
#[test]
fn test_stats_writes_a_per_file_section_as_markdown() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("markdown")
        .arg("--per-file")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();

    let per_file = text
        .split_once("## Per file")
        .unwrap_or_else(|| panic!("no per-file section: {text}"))
        .1;
    for name in ["hello_clang", "hello_gcc"] {
        assert!(
            per_file.contains(name),
            "{name} is not in the section: {text}"
        );
    }
}

/// `--top` is the third table, and the one whose rows have an order that
/// means something: rank 1 is the most frequent element, so a sort that went
/// the other way would still produce a well-formed table.
#[test]
fn test_stats_ranks_the_most_frequent_elements_as_csv() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("csv")
        .arg("--top")
        .arg("3")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines[0], "ir,birthmark_type,rank,element,count");
    assert_eq!(lines.len(), 4, "expected a header and three ranks: {text}");

    let counts: Vec<u64> = lines[1..]
        .iter()
        .map(|l| l.rsplit(',').next().unwrap().parse().unwrap())
        .collect();
    assert!(
        counts.windows(2).all(|w| w[0] >= w[1]),
        "the ranks are not in descending order of count: {counts:?}"
    );
}

/// One CSV file holds one table, so the two options that each replace the
/// summary cannot both be given. The refusal names the formats that do take
/// both, rather than only saying no.
#[test]
fn test_stats_refuses_two_csv_tables_and_says_where_to_put_them() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("csv")
        .arg("--per-file")
        .arg("--top")
        .arg("3")
        .arg(&dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("--per-file")
                .and(predicate::str::contains("-f json"))
                .and(predicate::str::contains("-f markdown")),
        );
}

/// `-o` is the other half of the driver: with it the report goes to the file
/// and stdout stays empty, which is what lets `stats` be piped into something
/// else without the report arriving twice.
#[test]
fn test_stats_writes_to_the_named_file_and_not_to_stdout() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);
    let dest = temp_dir.path().join("stats.md");

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-o")
        .arg(&dest)
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let written = fs::read_to_string(&dest).expect("the report was not written");
    assert!(
        written.contains("# Birthmark statistics"),
        "the file does not hold the report: {written}"
    );
}

/// JSON is the third format, and the one an agent or a script reads, so its
/// shape is a promise rather than a rendering. It was as unexercised as the
/// other two: `to_json` was never called by anything.
///
/// `occurrences` is asserted null here and counted in the test below, because
/// that field is the one thing in the report that depends on the birthmark's
/// shape rather than on its size: a sequence has no counts to total.
#[test]
fn test_stats_reports_the_groups_as_json() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).expect("the report is not JSON");

    let groups = json["groups"].as_array().expect("no groups array");
    assert_eq!(
        groups.len(),
        1,
        "two files of one type are one group: {json}"
    );
    assert_eq!(groups[0]["ir"], "ghidra-pcode");
    assert_eq!(groups[0]["birthmark_type"], "op-seq");
    assert_eq!(groups[0]["files"], 2);
    assert_eq!(groups[0]["elements"]["min"], 4);
    assert!(
        groups[0]["occurrences"].is_null(),
        "a sequence has no occurrence total: {json}"
    );
    assert!(json["skipped"].as_array().unwrap().is_empty());
}

/// A `freq` birthmark counts its elements, so the report has a total of those
/// counts where a sequence has none. That branch is chosen by the birthmark's
/// shape, and every test above uses `op-seq`, so nothing reached it.
#[test]
fn test_stats_totals_the_counts_of_a_freq_birthmark() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("extract")
        .arg("-d")
        .arg(&dir)
        .arg("-b")
        .arg("op-freq")
        .arg("testdata/lifted/pcodes/hello_clang.json")
        .assert()
        .success();

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let group = &json["groups"][0];

    assert_eq!(group["birthmark_type"], "op-freq");
    let vocabulary = group["vocabulary"].as_u64().expect("no vocabulary");
    let occurrences = group["occurrences"]
        .as_u64()
        .unwrap_or_else(|| panic!("a freq birthmark has an occurrence total: {json}"));
    assert!(
        occurrences >= vocabulary,
        "fewer occurrences than distinct elements: {occurrences} < {vocabulary}"
    );
}

/// A directory contributes the files directly inside it, and `-r` the ones
/// below as well. Asserted as the difference between the two runs over one
/// tree, because either alone would pass on a scan that ignored the flag.
#[test]
fn test_stats_descends_only_when_asked() {
    let temp_dir = tempdir().unwrap();
    let flat = temp_dir.path().join("birthmarks");
    birthmarks_in(&flat);

    let nested = temp_dir.path().join("nested");
    let deeper = nested.join("deeper");
    fs::create_dir_all(&deeper).unwrap();
    for entry in fs::read_dir(&flat).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap();
        let to = if name.to_string_lossy().starts_with("hello_gcc") {
            deeper.join(name)
        } else {
            nested.join(name)
        };
        fs::copy(&path, &to).unwrap();
    }

    let files_seen = |recursive: bool| {
        let mut cmd = Command::cargo_bin("oinkie").unwrap();
        cmd.arg("stats").arg("-f").arg("json");
        if recursive {
            cmd.arg("-r");
        }
        let out = cmd
            .arg(&nested)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
        json["groups"][0]["files"].as_u64().unwrap()
    };

    assert_eq!(files_seen(false), 1, "the subdirectory was read without -r");
    assert_eq!(files_seen(true), 2, "-r did not reach the subdirectory");
}

/// A file that does not read as a birthmark is skipped rather than fatal, and
/// counted rather than silent -- so a summary never covers fewer files than it
/// was given without saying so. The reason names the file, which is what makes
/// the report actionable instead of merely honest.
#[test]
fn test_stats_counts_what_it_could_not_read() {
    let temp_dir = tempdir().unwrap();
    let dir = temp_dir.path().join("birthmarks");
    birthmarks_in(&dir);
    fs::write(dir.join("junk.json"), r#"{"not":"a birthmark"}"#).unwrap();

    let out = Command::cargo_bin("oinkie")
        .unwrap()
        .arg("stats")
        .arg("-f")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();

    let skipped = json["skipped"].as_array().expect("no skipped array");
    assert_eq!(
        skipped.len(),
        1,
        "the unreadable file was not counted: {json}"
    );
    assert!(
        skipped[0]["path"].as_str().unwrap().ends_with("junk.json"),
        "the skip does not name the file: {json}"
    );
    assert_eq!(
        json["groups"][0]["files"], 2,
        "the readable files were affected: {json}"
    );
}

/// A lift that came back with no functions succeeds, and warns.
///
/// The quieter counterpart of the test above. `EmptyLifter` writes a file
/// oinkie reads without complaint and that holds nothing, which is what a
/// Ghidra missing its decompiler native binary produces (#126) -- and also
/// what a binary with genuinely no functions would produce. oinkie cannot tell
/// the two apart, so it says what it sees and goes on.
///
/// End to end because the warning is the whole deliverable: it has to reach a
/// person's terminal, which no unit test of the message can show.
#[test]
#[serial(ghidra)]
fn test_a_lift_that_found_no_functions_warns_and_succeeds() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    Command::cargo_bin("oinkie")
        .unwrap()
        .arg("lift")
        .arg("--script")
        .arg("testdata/scripts/EmptyLifter.java")
        .arg("-d")
        .arg(&dest)
        .arg("testdata/bin/hello_clang")
        .assert()
        .success()
        // the binary the person asked about, with the colon that follows it
        .stderr(predicate::str::contains("testdata/bin/hello_clang:"))
        .stderr(predicate::str::contains("no functions"))
        // both readings, because oinkie cannot choose between them
        .stderr(predicate::str::contains("really has no functions"))
        .stderr(predicate::str::contains("the tool found none"));

    // and the file is there to be used, which is what "succeeds" has to mean
    assert!(
        dest.join("hello_clang.json").exists(),
        "the lift warned and then kept nothing"
    );
}
