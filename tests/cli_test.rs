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
        .arg("-l")
        .arg("ghidra")
        .arg("testdata/hello_world/bin/hello_clang")
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
        .arg("testdata/hello_world/bin/hello_clang")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "/oinkie-no-such-ghidra/support/analyzeHeadless",
        ));
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
        .arg("testdata/hello_world/pcodes/hello_gcc.json")
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
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
        "testdata/hello_world/pcodes/hello_clang.json",
        "testdata/hello_world/pcodes/hello_gcc.json",
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
        .arg("testdata/hello_world/pcodes/hello_gcc.json")
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
        .arg("testdata/hello_world/pcodes/hello_gcc.json")
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
fn test_reaggregate_command() {
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
        .arg("testdata/hello_world/pcodes/hello_clang.json")
        .arg("testdata/hello_world/pcodes/hello_gcc.json")
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

    // Reaggregate
    let dest_file = temp_dir.path().join("reaggregate.csv");
    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    cmd.arg("reaggregate")
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
    let input = fs::canonicalize("testdata/hello_world/bin/hello_clang").unwrap();
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
        .arg("testdata/quoted_names/bin/udl")
        .assert()
        .success();

    let out_file = dest.join("udl.json");
    // through oinkie's own reader rather than a parser of the test's
    // choosing: this is the operation that used to fail
    oinkie::prelude::AnyProgram::load(&out_file)
        .expect("oinkie cannot read the file it just wrote");

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
