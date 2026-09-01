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
