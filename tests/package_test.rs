//! What has to survive `exclude` for the published crate to build.
//!
//! `Cargo.toml`'s `exclude` decides what is left out of the `.crate`, and a
//! file compiled in with `include_str!` that it drops produces a crate that
//! fails to build for whoever ran `cargo add` while building perfectly here --
//! a broken reference no link check can see, found by a stranger.
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
    let mut expected: Vec<String> = COMPILED_IN.iter().map(ToString::to_string).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the list in this test no longer matches the include_str! calls in src/"
    );
}

/// Nothing that is compiled in is left out of the `.crate`.
///
/// Asked of `cargo package --list` rather than of the `exclude` list, because
/// `exclude` takes gitignore globs and this was first written as a prefix
/// check. That check passed while `"/assets/lifters/**/*.py"` dropped both
/// Python lifting scripts -- a rule no `starts_with` can see, and exactly the
/// kind someone reaches for when trimming a package.
///
/// Reimplementing cargo's matching would be a second implementation to keep in
/// step with the first. Asking cargo costs about a tenth of a second and cannot
/// disagree with it.
#[test]
fn test_the_package_carries_every_compiled_in_file() {
    let output = std::process::Command::new(env!("CARGO"))
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("cargo package --list did not run");
    assert!(
        output.status.success(),
        "cargo package --list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let listed: Vec<&str> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(str::trim)
        .collect();

    for file in COMPILED_IN {
        assert!(
            Path::new(file).exists(),
            "{file} is compiled in but is not there"
        );
        assert!(
            listed.contains(&file),
            "{file} is compiled in but `exclude` keeps it out of the package, so the \
             published crate would not build. Listed: {} files",
            listed.len()
        );
    }
}

/// The library does not reach for what only the CLI needs.
///
/// `cli` is a default feature, so a build here never notices if `src/` starts
/// using one of them -- `cargo add oinkie --no-default-features` would be the
/// only thing that failed, and only for someone else. Grepping is enough to
/// say so, and costs nothing.
#[test]
fn test_the_library_does_not_use_the_cli_only_dependencies() {
    for crate_name in ["env_logger", "indicatif"] {
        let module = crate_name.replace('-', "_");
        for file in walk(Path::new("src")) {
            let text = std::fs::read_to_string(&file).unwrap();
            for (n, line) in text.lines().enumerate() {
                assert!(
                    !line.contains(&format!("{module}::")),
                    "{}:{}: the library uses {crate_name}, which is behind the `cli` feature -- \
                     either move it out of that feature or stop using it here",
                    file.display(),
                    n + 1
                );
            }
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
