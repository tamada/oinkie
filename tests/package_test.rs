//! What has to survive `exclude` for the published crate to build.
//!
//! `Cargo.toml`'s `exclude` decides what is left out of the `.crate`, and a
//! file compiled in with `include_str!` that it drops produces a crate that
//! fails to build for whoever ran `cargo add` while building perfectly here.
//! That is #91 again -- a reference no link check can see -- in the one place
//! where the person who finds it is a stranger.
//!
//! These assertions are about paths rather than content, so they cost nothing
//! and run everywhere.

use std::path::Path;

/// Every file `src/` pulls in at compile time, and the reason it is listed.
const COMPILED_IN: [&str; 3] = [
    // src/ghidra/lifter.rs
    "assets/lifters/ghidra/scripts/HighPCodeLifter.java",
    // src/binaryninja/lifter.rs
    "assets/lifters/binaryninja/scripts/BnilLifter.py",
    // src/ida/lifter.rs
    "assets/lifters/ida/scripts/MicrocodeLifter.py",
];

/// The list above is maintained by hand, so this says it has not fallen behind
/// the code: every `include_str!` in `src/` has to name a file on it.
#[test]
fn test_every_compiled_in_file_is_listed_here() {
    let mut found = Vec::new();
    for entry in walk(Path::new("src")) {
        let text = std::fs::read_to_string(&entry).unwrap();
        for line in text.lines() {
            let Some(rest) = line.split_once("include_str!(\"") else {
                continue;
            };
            let Some((relative, _)) = rest.1.split_once('"') else {
                continue;
            };
            let resolved = entry.parent().unwrap().join(relative);
            let resolved = resolved
                .canonicalize()
                .unwrap_or_else(|e| panic!("{}: {relative}: {e}", entry.display()));
            let root = std::env::current_dir().unwrap().canonicalize().unwrap();
            found.push(
                resolved
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    found.sort();
    let mut expected: Vec<String> = COMPILED_IN.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the list in this test no longer matches the include_str! calls in src/"
    );
}

/// Nothing in `Cargo.toml`'s `exclude` covers a file that has to ship.
///
/// Checked against the manifest rather than by packaging, so it runs without
/// `cargo package` and fails with the offending rule named.
#[test]
fn test_exclude_does_not_drop_a_compiled_in_file() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let block = manifest
        .split_once("exclude = [")
        .expect("Cargo.toml has no exclude list")
        .1
        .split_once(']')
        .unwrap()
        .0;
    let rules: Vec<&str> = block
        .lines()
        .filter_map(|l| l.trim().trim_end_matches(',').strip_prefix('"'))
        .filter_map(|l| l.strip_suffix('"'))
        .collect();
    assert!(!rules.is_empty(), "no exclude rules were parsed");

    for file in COMPILED_IN {
        assert!(
            Path::new(file).exists(),
            "{file} is compiled in but is not there"
        );
        for rule in &rules {
            let prefix = rule.trim_start_matches('/');
            assert!(
                !file.starts_with(prefix),
                "exclude rule {rule:?} drops {file}, which is compiled in"
            );
        }
    }
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}
