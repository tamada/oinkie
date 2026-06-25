use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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
fn test_lift_command() {
    let temp_dir = tempdir().unwrap();
    let dest = temp_dir.path().join("lifted");

    let mut cmd = Command::cargo_bin("oinkie").unwrap();
    let result = cmd.arg("lift")
        .arg("-d")
        .arg(&dest)
        .arg("-l")
        .arg("ghidra")
        .arg("testdata/hello_world/bin/hello_clang")
        .assert();

    // The lift command will fail if Ghidra is not installed.
    // In environments where Ghidra is not installed, it will exit with a non-zero status code.
    // If it succeeds, we check that the JSON file was generated.
    if String::from_utf8_lossy(&result.get_output().stderr).contains("GHIDRA_HOME not found") {
        println!("Skipping lift test as Ghidra is not installed.");
    } else {
        result.success();
        let out_file = dest.join("hello_clang.json");
        assert!(out_file.exists(), "hello_clang.json was not generated");
    }
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
    let entries: Vec<_> = fs::read_dir(&dest).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(entries.len(), 2, "Expected 2 birthmark files to be generated");
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

    let entries: Vec<_> = fs::read_dir(&birthmarks_dir).unwrap().map(|e| e.unwrap().path()).collect();
    
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

    let sim_entries: Vec<_> = fs::read_dir(&similarities_dir).unwrap().map(|e| e.unwrap().path()).collect();
    assert!(!sim_entries.is_empty(), "Expected similarity files to be generated");
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
        
    let sim_entries: Vec<_> = fs::read_dir(&similarities_dir).unwrap().map(|e| e.unwrap().path()).collect();
    assert!(!sim_entries.is_empty(), "Expected similarity files to be generated");
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

    let entries: Vec<_> = fs::read_dir(&birthmarks_dir).unwrap().map(|e| e.unwrap().path()).collect();

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
