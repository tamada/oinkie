//! What the committed IDA Pro fixtures say about the reader.
//!
//! IDA Pro is licensed and cannot be installed on a CI runner, so nothing here
//! lifts — see `assets/lifters/README.md`. The fixtures under
//! `testdata/lifted/mcode/` were produced by `oinkie lift` on a machine that
//! has it, and reading them back is what keeps the reader honest where the
//! lifter cannot run.
//!
//! They carry a repository-relative `path`, edited after lifting the way
//! `lifted/pcodes/` already is: a fresh lift records the absolute path it
//! canonicalised, which is particular to the machine that ran it.

use oinkie::Program;
use oinkie::birthmarks::BirthmarkType;
use oinkie::compare::{Aggregator, Algorithm, Comparator};
use oinkie::extract::Extractor;
use oinkie::lift::Ir;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new("testdata/lifted/mcode").join(name)
}

#[test]
fn test_the_fixture_reads_back_as_the_representation_it_names() {
    let p = Program::load(&fixture("hello_clang.json")).unwrap();
    assert_eq!(p.ir(), Ir::IdaMicrocode);
    assert_eq!(p.name(), "hello_clang");
}

/// The failure this exists for is silent, and it happened twice.
///
/// The microcode renders a global as `$name`, and the symbol table was first
/// keyed by the resolved name instead, so it was empty. Separately, the
/// optimiser folds a call into whatever consumes its result, so a call can sit
/// inside another instruction's operand; walking only the block's list missed
/// it.
///
/// Neither failed loudly. `m_call` is still a call wherever it is, so the check
/// that refuses a program in which nothing is a call passed in the first case,
/// and every `fc-*` birthmark simply came out empty — two of which score as a
/// perfect match. So this asserts that the call is *resolved*, not that one is
/// present.
#[test]
fn test_the_call_in_the_fixture_is_resolved_to_its_name() {
    let p = Program::load(&fixture("hello_clang.json")).unwrap();
    let b = Extractor::new(BirthmarkType::try_from("fc-set").unwrap())
        .extract(&p)
        .unwrap();
    let calls: Vec<String> = b
        .functions()
        .iter()
        .flat_map(|e| e.ops().map(|s| s.to_string()))
        .collect();
    assert!(
        calls.iter().any(|c| c == "_printf"),
        "no call resolved to _printf: {calls:?}"
    );
}

/// The microcode is not P-Code, and mixing them would measure the decompilers
/// rather than the programs.
#[test]
fn test_the_microcode_refuses_to_be_compared_with_p_code() {
    let mcode = Program::load(&fixture("hello_clang.json")).unwrap();
    let pcode = Program::load(Path::new("testdata/lifted/pcodes/hello_clang.json")).unwrap();
    let err = match Comparator::from(&Algorithm::Jaccard).compare_programs(
        &mcode,
        &pcode,
        &Aggregator::default(),
    ) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("microcode and P-Code were compared"),
    };
    assert!(
        err.contains("ida-microcode") && err.contains("ghidra-pcode"),
        "the refusal does not name both representations: {err}"
    );
}

#[test]
fn test_two_programs_in_the_microcode_compare() {
    let a = Program::load(&fixture("hello_clang.json")).unwrap();
    let b = Program::load(&fixture("hello_gcc.json")).unwrap();
    let c = Comparator::from(&Algorithm::Jaccard)
        .compare_programs(&a, &b, &Aggregator::default())
        .unwrap();
    let s = c.similarity();
    assert!((0.0..=1.0).contains(&s), "scored outside [0, 1]: {s}");
}
