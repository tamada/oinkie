//! What the committed IDA Pro fixtures say about the reader.
//!
//! IDA Pro is licensed and cannot be installed on a CI runner, so nothing here
//! lifts — see `assets/lifters/README.md`. The fixtures under
//! `testdata/lifted/mcode_*` were produced by `oinkie lift` on a machine that
//! has it, and reading them back is what keeps the reader honest where the
//! lifter cannot run.
//!
//! They carry a repository-relative `path`, edited after lifting the way
//! `lifted/pcodes/` already is: a fresh lift records the absolute path it
//! canonicalised, which is particular to the machine that ran it.

use oinkie::prelude::*;
use std::path::{Path, PathBuf};

const MATURITIES: [(&str, Ir); 8] = [
    ("generated", Ir::IdaMicrocodeGenerated),
    ("preoptimized", Ir::IdaMicrocodePreoptimized),
    ("locopt", Ir::IdaMicrocodeLocopt),
    ("calls", Ir::IdaMicrocodeCalls),
    ("glbopt1", Ir::IdaMicrocodeGlbopt1),
    ("glbopt2", Ir::IdaMicrocodeGlbopt2),
    ("glbopt3", Ir::IdaMicrocodeGlbopt3),
    ("lvars", Ir::IdaMicrocodeLvars),
];

fn fixture(maturity: &str, name: &str) -> PathBuf {
    Path::new("testdata/lifted")
        .join(format!("mcode_{maturity}"))
        .join(name)
}

#[test]
fn test_each_maturity_reads_back_as_the_representation_it_names() {
    for (maturity, ir) in MATURITIES {
        let p = AnyProgram::load(&fixture(maturity, "hello_clang.json"))
            .unwrap_or_else(|e| panic!("{maturity}: {e}"));
        assert_eq!(p.ir(), ir, "{maturity} loaded as the wrong representation");
        assert_eq!(p.name(), "hello_clang");
    }
}

/// The failure this exists for is silent, and it happened: the microcode
/// renders a global as `$name`, the symbol table was first keyed by the
/// resolved name instead, and every fc-* birthmark came out empty. Nothing
/// caught it — `m_call` is still a call, so the check that refuses a program
/// in which nothing is a call passes — and two empty birthmarks score as a
/// perfect match.
#[test]
fn test_every_maturity_resolves_the_call_that_is_there() {
    for (maturity, _) in MATURITIES {
        let p = AnyProgram::load(&fixture(maturity, "hello_clang.json")).unwrap();
        let b = Extractor::new(BirthmarkType::try_from("fc-set").unwrap())
            .extract_any(&p)
            .unwrap_or_else(|e| panic!("{maturity}: {e}"));
        let calls: Vec<String> = b
            .iter()
            .flat_map(|e| e.ops().map(|s| s.to_string()))
            .collect();
        assert!(
            calls.iter().any(|c| c == "_printf"),
            "{maturity} resolved no call to _printf: {calls:?}"
        );
    }
}

/// Two maturities of one program are two representations, and mixing them
/// would measure the decompiler's pipeline rather than the programs.
#[test]
fn test_two_maturities_of_the_same_program_refuse_to_be_compared() {
    let generated = AnyProgram::load(&fixture("generated", "hello_clang.json")).unwrap();
    let lvars = AnyProgram::load(&fixture("lvars", "hello_clang.json")).unwrap();
    let err = match Comparator::from(&Algorithm::Jaccard).compare_any(
        &generated,
        &lvars,
        &Aggregator::default(),
    ) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("two maturities were compared"),
    };
    assert!(
        err.contains("ida-microcode-generated") && err.contains("ida-microcode-lvars"),
        "the refusal does not name both representations: {err}"
    );
}

#[test]
fn test_a_maturity_compares_with_itself() {
    for (maturity, _) in MATURITIES {
        let a = AnyProgram::load(&fixture(maturity, "hello_clang.json")).unwrap();
        let b = AnyProgram::load(&fixture(maturity, "hello_gcc.json")).unwrap();
        let c = Comparator::from(&Algorithm::Jaccard)
            .compare_any(&a, &b, &Aggregator::default())
            .unwrap_or_else(|e| panic!("{maturity}: {e}"));
        let s = c.similarity();
        assert!(
            (0.0..=1.0).contains(&s),
            "{maturity} scored outside [0, 1]: {s}"
        );
    }
}

/// The pipeline rewrites the microcode, so the maturities are not copies of
/// one another. Asserting that at least one pair differs says the maturity
/// argument reaches IDA at all — every fixture being identical is what a
/// `--maturity` that was quietly ignored would look like.
#[test]
fn test_the_maturities_are_not_all_the_same_program() {
    let first = std::fs::read_to_string(fixture("generated", "hello_clang.json")).unwrap();
    let last = std::fs::read_to_string(fixture("lvars", "hello_clang.json")).unwrap();
    assert_ne!(
        first, last,
        "generated and lvars are byte-identical, so the maturity was ignored"
    );
}
