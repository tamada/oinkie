//! What the committed Binary Ninja fixtures say about the reader.
//!
//! Binary Ninja is licensed and cannot be installed on a CI runner, so
//! nothing here lifts. The fixtures under `testdata/lifted/bnil_*` were
//! produced by `oinkie lift` on a machine that has it, and reading them back
//! is what keeps the reader honest where the lifter cannot run.
//!
//! The fixtures carry a repository-relative `path`, edited after lifting the
//! way `lifted/pcodes/` already is: a fresh lift records the absolute path it
//! canonicalised, which is particular to the machine that ran it.

use oinkie::Program;
use oinkie::birthmarks::BirthmarkType;
use oinkie::compare::{Aggregator, Algorithm, Comparator};
use oinkie::extract::Extractor;
use oinkie::lift::Ir;
use std::path::{Path, PathBuf};

const LEVELS: [(&str, Ir); 3] = [
    ("llil", Ir::BinaryNinjaLlil),
    ("mlil", Ir::BinaryNinjaMlil),
    ("hlil", Ir::BinaryNinjaHlil),
];

fn fixture(level: &str, name: &str) -> PathBuf {
    Path::new("testdata/lifted")
        .join(format!("bnil_{level}"))
        .join(name)
}

#[test]
fn test_each_level_reads_back_as_the_representation_it_names() {
    for (level, ir) in LEVELS {
        let p = Program::load(&fixture(level, "hello_clang.json"))
            .unwrap_or_else(|e| panic!("{level}: {e}"));
        assert_eq!(p.ir(), ir, "{level} loaded as the wrong representation");
        assert_eq!(p.name(), "hello_clang");
    }
}

/// The failure this exists for is silent. If `is_call` recognised none of a
/// level's own opcodes, every `fc-*` birthmark would be empty, and two empty
/// birthmarks score 1.0 -- unrelated programs reported as identical.
///
/// `extract` refuses a program in which nothing is a call, so asking for
/// `fc-set` is the assertion. Asking for the elements as well says the
/// symbol table was keyed the way this level renders a callee, which is the
/// other half and differs between the three.
#[test]
fn test_every_level_finds_the_call_that_is_there() {
    for (level, _) in LEVELS {
        let p = Program::load(&fixture(level, "hello_clang.json")).unwrap();
        let b = Extractor::new(BirthmarkType::try_from("fc-set").unwrap())
            .extract(&p)
            .unwrap_or_else(|e| panic!("{level}: {e}"));
        let calls: Vec<String> = b
            .elements()
            .iter()
            .flat_map(|e| e.ops().map(|s| s.to_string()))
            .collect();
        assert!(
            calls.iter().any(|c| c == "_printf"),
            "{level} did not find the call to _printf: {calls:?}"
        );
    }
}

/// A representation is not comparable with another, and these three are the
/// first pairs that can actually reach the refusal: until Binary Ninja could
/// be read, only one representation existed.
#[test]
fn test_two_levels_of_the_same_program_refuse_to_be_compared() {
    let llil = Program::load(&fixture("llil", "hello_clang.json")).unwrap();
    let hlil = Program::load(&fixture("hlil", "hello_clang.json")).unwrap();
    let err = match Comparator::from(&Algorithm::Jaccard).compare_programs(
        &llil,
        &hlil,
        &Aggregator::default(),
    ) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("LLIL and HLIL were compared"),
    };
    assert!(
        err.contains("binary-ninja-llil") && err.contains("binary-ninja-hlil"),
        "the refusal does not name both representations: {err}"
    );
}

#[test]
fn test_a_level_compares_with_itself() {
    for (level, _) in LEVELS {
        let a = Program::load(&fixture(level, "hello_clang.json")).unwrap();
        let b = Program::load(&fixture(level, "hello_gcc.json")).unwrap();
        let c = Comparator::from(&Algorithm::Jaccard)
            .compare_programs(&a, &b, &Aggregator::default())
            .unwrap_or_else(|e| panic!("{level}: {e}"));
        let s = c.similarity();
        assert!(
            (0.0..=1.0).contains(&s),
            "{level} scored outside [0, 1]: {s}"
        );
    }
}
