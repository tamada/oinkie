//! Detects software theft by comparing birthmarks extracted from binaries.
//!
//! The root holds only what every use touches: [`Program`], [`Error`] and
//! [`Result`]. Everything else is reached by the module it belongs to, and by
//! that path alone:
//!
//! - [`lift`] -- turning a binary into a lifted program, through a tool;
//! - [`extract`] -- extracting a birthmark from a lifted program;
//! - [`compare`] -- comparing two birthmarks;
//! - [`birthmarks`] -- what a birthmark is, and what it holds.
//!
//! The terms -- birthmark, function, element, similarity -- are the ones the
//! [glossary](https://tamada.github.io/oinkie/glossary/) defines.
//!
//! # Example
//!
//! Two lifted programs, compared under one analysis: extract the analysis's
//! birthmark from each, then compare the birthmarks with its algorithm.
//!
//! ```
//! use std::path::Path;
//!
//! use oinkie::Program;
//! use oinkie::birthmarks::AnalysisType;
//! use oinkie::compare::Aggregator;
//! use oinkie::extract::Extractor;
//!
//! # fn main() -> oinkie::Result<()> {
//! let analysis = AnalysisType::try_from("op-set-jaccard")?;
//! let extractor = Extractor::new(analysis.birthmark().clone());
//! let a = extractor.extract(&Program::load(Path::new("testdata/lifted/pcodes/hello_clang.json"))?)?;
//! let b = extractor.extract(&Program::load(Path::new("testdata/lifted/pcodes/udl.json"))?)?;
//!
//! let comparison = analysis
//!     .comparator()
//!     .compare_birthmarks(&a, &b, &Aggregator::Hungarian)?;
//! let similarity = comparison.similarity();
//! assert!((0.0..=1.0).contains(&similarity));
//! # Ok(())
//! # }
//! ```

// Every public item is documented, and stays so: CI runs clippy with
// -D warnings, which makes this an error there (#173).
#![warn(missing_docs)]

use std::path::PathBuf;

use ndarray::ShapeError;

use crate::birthmarks::BirthmarkType;

mod binaryninja;
pub mod birthmarks;
pub mod compare;
pub mod extract;
mod ghidra;
mod ida;
pub mod lift;
mod program;

pub use crate::program::Program;

/// What every fallible function in this crate returns.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong in oinkie.
///
/// Each variant's message, its `Display`, is written for a person, and names
/// the file or the value it is about. [`Error::is_caller_fault`] says whether
/// asking differently could have avoided it.
///
/// The messages live on the variants rather than in a `Display` match, so
/// that adding a variant and deciding how it reads are the same edit.
///
/// The wrapped errors are `#[source]` but not `#[from]`. `Io` and `Json`
/// carry the path beside the error — which path failed is the useful half —
/// so they could not be `#[from]` anyway, and for the rest an explicit
/// `map_err(Error::Csv)` at the call site says more than a conversion hidden
/// inside a `?`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Several failures from one batch, one per input that failed and in the
    /// inputs' order, shown numbered from 1. [`Error::vec_result_to_result_vec`]
    /// makes it.
    #[error("{}", render_group(.0))]
    Array(Vec<Self>),
    /// A birthmark type name that does not parse, as it was given: an unknown
    /// family, or a k-gram of size 0.
    #[error("{0}: unknown birthmark type")]
    BirthmarkType(String),
    /// A CSV record could not be read or written.
    #[error("CSV error: {0}")]
    Csv(#[source] csv::Error),
    /// A birthmark shape paired with an algorithm that does not operate on it.
    #[error("{}", render_incompatible(.0, .1))]
    IncompatibleAnalysis(BirthmarkType, crate::compare::Algorithm),
    /// Two birthmarks lifted to different intermediate representations.
    #[error(
        "cannot compare {0} against {1}: the two are lifted to different intermediate representations, whose operation vocabularies do not correspond"
    )]
    IrMismatch(crate::lift::Ir, crate::lift::Ir),
    /// A lifter reported success but wrote a file that cannot be read back.
    ///
    /// Named for the binary as well as for the JSON, because the binary is
    /// what the caller asked about and the JSON is a file they have never
    /// seen. That the lifter reported success is the part that points at the
    /// script rather than at the binary: `analyzeHeadless` exits 0 whether or
    /// not its post-script threw, so a script that writes malformed output is
    /// indistinguishable from one that works until something reads what it
    /// wrote.
    ///
    /// The output's path is not repeated here: every error
    /// [`crate::Program::load`] can return carries it already, and
    /// naming it twice put the same long path in the message twice.
    #[error(
        "{binary}: the lifter reported success, but what it wrote cannot be read back: {cause}",
        binary = .0.display(),
        cause = .1
    )]
    UnreadableOutput(PathBuf, #[source] Box<Error>),
    /// A lifted file naming a representation this build cannot read.
    #[error("invalid pcode: {0}")]
    InvalidPcode(u32),
    /// An I/O failure on a path the caller gave: a file to read, a destination
    /// to write, a script or a working directory they named. A path oinkie
    /// chose itself is [`Error::ToolIo`] instead, and the difference is what
    /// [`Error::is_caller_fault`] answers from.
    #[error("IO error for {path}: {cause}", path = .0.display(), cause = .1)]
    Io(PathBuf, #[source] std::io::Error),
    /// An I/O failure on a path oinkie chose rather than one the caller gave:
    /// starting a lifting tool it found, or the scratch files it runs it with.
    ///
    /// Reads exactly like [`Error::Io`], since to a person at a terminal the
    /// path and the cause are what matter. It is a separate variant because
    /// whose path it was decides whether asking differently could have helped.
    #[error("IO error for {path}: {cause}", path = .0.display(), cause = .1)]
    ToolIo(PathBuf, #[source] std::io::Error),
    /// The file at the path is not JSON of the shape expected -- a lifted
    /// program, a birthmark -- or could not be written as JSON.
    #[error("{path}: JSON error: {cause}", path = .0.display(), cause = .1)]
    Json(PathBuf, #[source] serde_json::Error),
    /// The assignment the `hungarian` aggregator solves could not be solved.
    #[error("LapJV error: {0}")]
    LapJV(#[source] lapjv::LapJVError),
    /// Two birthmarks of different types were compared. A birthmark compares
    /// only with one of its own type.
    #[error("Mismatched birthmark types: {0} and {1}")]
    Mismatch(BirthmarkType, BirthmarkType),
    /// Something that could not be read or done, with the reason: an
    /// aggregator name, a file's contents, a lift that wrote nothing. A
    /// catch-all, which is why [`Error::is_caller_fault`] does not count it
    /// as the caller's.
    #[error("Parse error: {0}")]
    Parse(String),
    /// Text that should have been a decimal number: the text, and why it is
    /// not one.
    #[error("{0}: Parse float error {1}")]
    ParseFloat(String, #[source] std::num::ParseFloatError),
    /// Text that should have been a whole number: the text, and why it is not
    /// one.
    #[error("{0}: Parse int error {1}")]
    ParseInt(String, #[source] std::num::ParseIntError),
    /// A matrix could not be built in the shape asked for.
    #[error("Shape error: {0}")]
    ShapeError(#[source] ShapeError),
    /// A lifting tool's installation could not be found: none was given, its
    /// environment variable is unset, and it is in none of the usual places.
    ///
    /// Structured rather than a sentence, so that whoever shows it can say how
    /// to supply one in their own terms -- a flag, a setting -- instead of the
    /// library naming a way that only one caller has.
    #[error("{}", render_tool_not_found(.tool, .env, .candidates))]
    ToolNotFound {
        /// The tool, as a person names it: `Ghidra`.
        tool: &'static str,
        /// The environment variable that names its home: `GHIDRA_HOME`.
        env: &'static str,
        /// The usual installation directories that were looked in.
        candidates: &'static [&'static str],
    },
}

/// What a missing installation says without naming how a caller supplies one.
fn render_tool_not_found(tool: &str, env: &str, candidates: &[&str]) -> String {
    let looked_in = if candidates.is_empty() {
        String::new()
    } else {
        format!(", or install it in one of: {}", candidates.join(", "))
    };
    format!("{tool} not found. Give its home directory, set {env}{looked_in}")
}

/// Numbered from one, because this is read by someone counting which of their
/// inputs failed.
fn render_group(errs: &[Error]) -> String {
    let mut s = String::from("Multiple errors:");
    for (i, err) in errs.iter().enumerate() {
        s.push_str(&format!("\n  {}. {}", i + 1, err));
    }
    s
}

fn render_incompatible(bt: &BirthmarkType, algorithm: &crate::compare::Algorithm) -> String {
    let name = algorithm.name();
    format!(
        "{bt}-{name}: {name} operates on {}; use {}-{name}",
        algorithm.shape().description(),
        bt.with_shape(algorithm.shape())
    )
}

impl Error {
    /// Whether the caller could have avoided this by asking differently: a
    /// name, a pairing or a number they supplied, or a file they named.
    ///
    /// Decided here, by an exhaustive match, because this is the one place the
    /// match can stay exhaustive. `Error` is `#[non_exhaustive]`, so a crate
    /// that matches on it has to have a wildcard arm, and a new variant would
    /// fall into that arm unclassified. Inside the crate that defines it, a
    /// new variant stops the build here until someone decides which it is.
    pub fn is_caller_fault(&self) -> bool {
        match self {
            // A name, a pairing or a number that the caller supplied.
            Error::BirthmarkType(_)
            | Error::IncompatibleAnalysis(_, _)
            | Error::Mismatch(_, _)
            | Error::IrMismatch(_, _)
            | Error::ParseFloat(_, _)
            | Error::ParseInt(_, _) => true,

            // A file the caller named, which they can name differently.
            Error::Io(_, _) | Error::Json(_, _) => true,

            // Something went wrong inside, or in a file oinkie itself
            // produced; or in the machine it runs on, which no argument can
            // change. A missing tool was never the caller's fault either, back
            // when it arrived as a `Parse`.
            //
            // `ToolIo` is on a path oinkie chose: the tool it found, or a
            // scratch file. The tool's path can come from a home the caller
            // gave, but as often from the environment or a usual location,
            // and nothing in the error says which -- so, as with `Parse`, it
            // does not claim the caller is at fault.
            Error::ToolNotFound { .. }
            | Error::ToolIo(_, _)
            | Error::Csv(_)
            | Error::InvalidPcode(_)
            | Error::LapJV(_)
            | Error::ShapeError(_)
            | Error::UnreadableOutput(_, _) => false,

            // `Parse` is a catch-all carrying a string, and the strings it
            // carries come from both sides: "Invalid aggregator" is the
            // caller's, while "could not start N lift jobs" is not. Nothing in
            // the variant says which.
            //
            // So it does not claim the caller is at fault. Telling a caller it
            // asked wrongly when it did not is the more expensive mistake --
            // it will try different arguments, repeatedly, against something
            // no argument can fix. A caller wanting a definite answer for a
            // value it took (the aggregator is what reaches `Parse` that way)
            // should validate that value itself rather than rely on this.
            Error::Parse(_) => false,

            // A group is the caller's fault only if all of it is. One internal
            // failure in a batch makes the whole batch an internal failure,
            // since saying "you asked wrongly" about it would be wrong for
            // that one.
            Error::Array(errs) => errs.iter().all(Error::is_caller_fault),
        }
    }

    /// Every value, or every error: the results of a batch, one per input.
    ///
    /// All succeeding gives the values in the inputs' order. One failing gives
    /// its error as it is, and several give an [`Error::Array`] of them in the
    /// same order, so that no failure is dropped in favour of the first.
    pub fn vec_result_to_result_vec<T>(vec: Vec<Result<T>>) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut errs = Vec::new();
        for r in vec {
            match r {
                Ok(v) => results.push(v),
                Err(e) => errs.push(e),
            }
        }
        Self::error_or(results, errs)
    }

    pub(crate) fn error_or<T>(result: T, errs: Vec<Self>) -> Result<T> {
        if errs.is_empty() {
            Ok(result)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Self::Array(errs))
        }
    }
}

pub(crate) trait Op {
    /// returns the mnemonic of the operation, e.g., "ADD", "SUB", etc.
    fn mnemonic(&self) -> &str;

    /// returns the inputs of the operation, e.g., the source registers or memory locations.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no birthmark reads operands yet; each lifter's tests pin how its IR fills them"
        )
    )]
    fn inputs(&self) -> &[String];

    /// returns whether this operation transfers control to another function,
    /// which is what the `fc-*` birthmarks are built from.
    ///
    /// Each intermediate representation spells its call differently — P-Code
    /// writes `CALL`, the Hex-Rays microcode `m_call`, Binary Ninja's LLIL
    /// `LLIL_CALL` — and some name more than one. Deciding here rather than in
    /// [`crate::extract`] keeps that vocabulary with the lifter that owns
    /// it.
    ///
    /// This method deliberately has no default. A `false` default would let a
    /// new lifter compile while matching no operation at all, and an `fc-*`
    /// birthmark that matched nothing is not merely useless: two empty
    /// birthmarks score as a perfect match, so unrelated programs would be
    /// reported as identical. Requiring the method makes that a build failure
    /// instead of a wrong answer.
    fn is_call(&self) -> bool;

    /// returns the output of the operation, e.g., the destination register or memory location.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no birthmark reads results yet; each lifter's tests pin how its IR fills them"
        )
    )]
    fn ret(&self) -> Option<&str>;

    /// returns this operation's first operand rendered as the program's symbol
    /// table keys it, so that [`crate::program::TypedProgram::symbol`] can resolve
    /// it, or `None` when that operand cannot name a symbol.
    ///
    /// Callers choose which operations to ask — the only caller today asks
    /// calls, to build the `fc-*` birthmarks — so an implementation need not
    /// inspect the opcode itself.
    ///
    /// Returning `None` for a target no symbol could name is the part that
    /// matters. An indirect call through a register or a temporary is
    /// resolved at run time and has no name to find; a key that cannot match
    /// would be indistinguishable from a lookup that legitimately found
    /// nothing, which is how the `fc-*` family came to be silently empty
    /// before this method existed.
    ///
    /// The operand notation is the lifter's own — Ghidra writes
    /// `"(ram, 0x100000480, 8)"` while its symbol table is keyed
    /// `"0x100000480"` — so reconciling the two belongs with the lifter that
    /// produced both, not with the extractor that is generic over them.
    fn symbol_key(&self) -> Option<String>;
}

pub(crate) trait Iterable {
    type Item;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::Algorithm;
    use crate::lift::Ir;

    /// Adding a variant makes this stop compiling, which is the reminder to
    /// add it to `rendered_errors` below as well. There is no way to
    /// enumerate an enum's variants, so an exhaustive match is the closest a
    /// test can get to noticing that it has fallen behind.
    fn variant_name(e: &Error) -> &'static str {
        match e {
            Error::Array(_) => "Array",
            Error::BirthmarkType(_) => "BirthmarkType",
            Error::Csv(_) => "Csv",
            Error::IncompatibleAnalysis(_, _) => "IncompatibleAnalysis",
            Error::IrMismatch(_, _) => "IrMismatch",
            Error::UnreadableOutput(_, _) => "UnreadableOutput",
            Error::InvalidPcode(_) => "InvalidPcode",
            Error::Io(_, _) => "Io",
            Error::ToolIo(_, _) => "ToolIo",
            Error::Json(_, _) => "Json",
            Error::LapJV(_) => "LapJV",
            Error::Mismatch(_, _) => "Mismatch",
            Error::Parse(_) => "Parse",
            Error::ParseFloat(_, _) => "ParseFloat",
            Error::ParseInt(_, _) => "ParseInt",
            Error::ShapeError(_) => "ShapeError",
            Error::ToolNotFound { .. } => "ToolNotFound",
        }
    }

    /// One of each variant, paired with what it has to read as.
    ///
    /// The foreign errors are obtained rather than constructed — `csv::Error`
    /// and `lapjv::LapJVError` have no public constructor — so each is
    /// produced by the smallest operation that fails that way.
    ///
    /// Where a variant wraps one of them, the expectation is built from that
    /// error's own `to_string` rather than from its wording pasted in. What
    /// is being tested is this crate's half — the prefix, and that the inner
    /// message is carried at all — and pinning `serde_json`'s phrasing would
    /// turn a dependency bump into a test failure that says nothing.
    fn rendered_errors() -> Vec<(Error, String)> {
        let csv_err = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader("a,b\nc\n".as_bytes())
            .records()
            .nth(1)
            .unwrap()
            .unwrap_err();
        let lapjv_err = lapjv::lapjv(&ndarray::Array2::<f64>::zeros((2, 3))).unwrap_err();
        let json_err = serde_json::from_str::<i32>("nope").unwrap_err();
        let float_err = "x".parse::<f64>().unwrap_err();
        let int_err = "x".parse::<i32>().unwrap_err();
        let shape_err = ndarray::Array2::from_shape_vec((2, 2), vec![1.0]).unwrap_err();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let tool_io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");

        let (csv_msg, lapjv_msg, json_msg) = (
            csv_err.to_string(),
            lapjv_err.to_string(),
            json_err.to_string(),
        );
        let (float_msg, int_msg, shape_msg) = (
            float_err.to_string(),
            int_err.to_string(),
            shape_err.to_string(),
        );
        let io_msg = io_err.to_string();

        vec![
            (
                Error::Array(vec![
                    Error::Parse("first".to_string()),
                    Error::Parse("second".to_string()),
                ]),
                // numbered, and from one rather than zero: this is read by a
                // person counting which of their inputs failed
                "Multiple errors:\n  1. Parse error: first\n  2. Parse error: second".to_string(),
            ),
            (
                Error::BirthmarkType("nonsense".to_string()),
                "nonsense: unknown birthmark type".to_string(),
            ),
            (Error::Csv(csv_err), format!("CSV error: {csv_msg}")),
            (
                Error::IncompatibleAnalysis(BirthmarkType::OpSeq, Algorithm::Euclidean),
                "op-seq-euclidean: euclidean operates on frequency vectors; use op-freq-euclidean"
                    .to_string(),
            ),
            (
                Error::IrMismatch(Ir::GhidraPcode, Ir::IdaMicrocode),
                "cannot compare ghidra-pcode against ida-microcode: the two are lifted to different intermediate representations, whose operation vocabularies do not correspond".to_string(),
            ),
            (
                Error::UnreadableOutput(
                    PathBuf::from("bin/sample"),
                    Box::new(Error::Json(
                        PathBuf::from("pcodes/sample.json"),
                        serde_json::from_str::<i32>("nope").unwrap_err(),
                    )),
                ),
                format!(
                    "bin/sample: the lifter reported success, but what it wrote cannot be read back: pcodes/sample.json: JSON error: {json_msg}"
                ),
            ),
            (
                Error::InvalidPcode(9999),
                "invalid pcode: 9999".to_string(),
            ),
            (
                Error::Io(PathBuf::from("missing.json"), io_err),
                format!("IO error for missing.json: {io_msg}"),
            ),
            (
                // the same words as `Io`: whose path it was is not the reader's concern
                Error::ToolIo(PathBuf::from("ghidra/support/analyzeHeadless"), tool_io_err),
                format!("IO error for ghidra/support/analyzeHeadless: {io_msg}"),
            ),
            (
                Error::Json(PathBuf::from("broken.json"), json_err),
                format!("broken.json: JSON error: {json_msg}"),
            ),
            (Error::LapJV(lapjv_err), format!("LapJV error: {lapjv_msg}")),
            (
                Error::Mismatch(BirthmarkType::OpSeq, BirthmarkType::OpKgramSet(3)),
                "Mismatched birthmark types: op-seq and op-3gram-set".to_string(),
            ),
            (
                Error::Parse("something went wrong".to_string()),
                "Parse error: something went wrong".to_string(),
            ),
            (
                Error::ParseFloat("x".to_string(), float_err),
                format!("x: Parse float error {float_msg}"),
            ),
            (
                Error::ParseInt("x".to_string(), int_err),
                format!("x: Parse int error {int_msg}"),
            ),
            (Error::ShapeError(shape_err), format!("Shape error: {shape_msg}")),
            (
                Error::ToolNotFound {
                    tool: "Ghidra",
                    env: "GHIDRA_HOME",
                    candidates: &["/opt/ghidra", "/usr/local/ghidra"],
                },
                "Ghidra not found. Give its home directory, set GHIDRA_HOME, or install it in one of: /opt/ghidra, /usr/local/ghidra".to_string(),
            ),
        ]
    }

    /// `Parse` carries a string and nothing else, and those strings come from
    /// both sides. Since the variant cannot say which, it does not claim the
    /// caller is at fault -- no argument would fix this one.
    #[test]
    fn test_the_catch_all_does_not_claim_the_caller_is_at_fault() {
        let e = Error::Parse("could not start 4 lift jobs".to_string());
        assert!(!e.is_caller_fault());
    }

    #[test]
    fn test_a_bad_name_is_the_callers_fault_and_a_broken_file_of_ours_is_not() {
        assert!(Error::BirthmarkType("nonsense".to_string()).is_caller_fault());
        assert!(!Error::InvalidPcode(9999).is_caller_fault());
        let missing = Error::ToolNotFound {
            tool: "Ghidra",
            env: "GHIDRA_HOME",
            candidates: &[],
        };
        assert!(!missing.is_caller_fault(), "no argument installs a tool");
        let not_started = Error::ToolIo(
            PathBuf::from("ghidra/support/analyzeHeadless"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
        );
        assert!(
            !not_started.is_caller_fault(),
            "a path oinkie chose is not one the caller can name differently"
        );
    }

    /// A file the caller named is theirs to name differently, whether it is
    /// missing or is not the JSON it should be.
    #[test]
    fn test_a_file_the_caller_named_is_the_callers_fault() {
        let missing = Error::Io(
            PathBuf::from("no/such.json"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
        );
        assert!(missing.is_caller_fault());
        let json = serde_json::from_str::<i32>("nope").unwrap_err();
        assert!(Error::Json(PathBuf::from("bad.json"), json).is_caller_fault());
    }

    /// A group of nothing but the caller's own mistakes is still theirs.
    /// Asserted in both directions, or `all` could be inverted and only one
    /// of these would notice.
    #[test]
    fn test_a_group_of_the_callers_mistakes_is_the_callers_fault() {
        let e = Error::Array(vec![
            Error::BirthmarkType("nonsense".to_string()),
            Error::BirthmarkType("also nonsense".to_string()),
        ]);
        assert!(e.is_caller_fault());
    }

    /// One internal failure makes the batch internal. Reporting "you asked
    /// wrongly" for a group containing something the caller could not have
    /// avoided would be wrong about that one.
    #[test]
    fn test_one_internal_failure_makes_the_whole_group_internal() {
        let e = Error::Array(vec![
            Error::BirthmarkType("the caller's".to_string()),
            Error::UnreadableOutput(
                PathBuf::from("bin/sample"),
                Box::new(Error::Parse("ours".to_string())),
            ),
        ]);
        assert!(!e.is_caller_fault());
    }

    #[test]
    fn test_every_error_says_what_it_is() {
        for (err, expected) in rendered_errors() {
            assert_eq!(err.to_string(), expected, "{}", variant_name(&err));
        }
    }

    #[test]
    fn test_no_variant_is_in_the_table_twice() {
        let mut names = rendered_errors()
            .iter()
            .map(|(e, _)| variant_name(e))
            .collect::<Vec<_>>();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "a variant is covered twice: {names:?}");
    }

    /// The numbering is the part worth pinning. `Array` is what a run over
    /// many inputs produces, and "the third one failed" is the only thing the
    /// reader can act on.
    #[test]
    fn test_a_group_of_errors_numbers_its_children_from_one() {
        let e = Error::Array(vec![
            Error::Parse("a".to_string()),
            Error::Parse("b".to_string()),
            Error::Parse("c".to_string()),
        ]);
        let rendered = e.to_string();
        assert!(rendered.contains("\n  1. Parse error: a"), "{rendered}");
        assert!(rendered.contains("\n  3. Parse error: c"), "{rendered}");
        assert!(!rendered.contains("0."), "numbered from zero: {rendered}");
    }

    /// `impl std::error::Error for Error {}` was empty, so `source()` was
    /// `None` even for the variants holding a cause. Nothing called it —
    /// there was nothing to get (#62).
    #[test]
    fn test_a_wrapped_error_is_reachable_as_a_source() {
        use std::error::Error as _;

        let inner = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let inner_msg = inner.to_string();
        let e = Error::Io(PathBuf::from("locked.json"), inner);
        let source = e.source().expect("the io::Error is the cause");
        assert_eq!(source.to_string(), inner_msg);

        let e = Error::Json(
            PathBuf::from("broken.json"),
            serde_json::from_str::<i32>("nope").unwrap_err(),
        );
        assert!(e.source().is_some(), "the serde_json::Error is the cause");
    }

    /// A variant that is not wrapping anything has no cause to report, and
    /// saying otherwise would make a chain look deeper than it is.
    #[test]
    fn test_an_error_of_our_own_has_no_source() {
        use std::error::Error as _;

        assert!(Error::Parse("ours".to_string()).source().is_none());
        assert!(Error::InvalidPcode(1).source().is_none());
        assert!(
            Error::Array(vec![Error::Parse("child".to_string())])
                .source()
                .is_none()
        );
    }

    #[test]
    fn test_no_errors_is_not_an_error() {
        let r: Result<Vec<i32>> = Error::vec_result_to_result_vec(vec![Ok(1), Ok(2)]);
        assert_eq!(r.unwrap(), vec![1, 2]);
        assert_eq!(Error::error_or("kept", vec![]).unwrap(), "kept");
    }

    /// A single failure is reported as itself. Wrapping it would put
    /// "Multiple errors:" in front of one error, and the caller would have to
    /// unwrap a group to find out there was nothing to group.
    #[test]
    fn test_one_error_is_not_wrapped_in_a_group() {
        let r: Result<Vec<i32>> =
            Error::vec_result_to_result_vec(vec![Ok(1), Err(Error::Parse("only".to_string()))]);
        let err = r.unwrap_err();
        assert_eq!(variant_name(&err), "Parse");
        assert_eq!(err.to_string(), "Parse error: only");
    }

    #[test]
    fn test_several_errors_are_grouped_and_none_is_dropped() {
        let r: Result<Vec<i32>> = Error::vec_result_to_result_vec(vec![
            Err(Error::Parse("first".to_string())),
            Ok(1),
            Err(Error::Parse("second".to_string())),
        ]);
        let err = r.unwrap_err();
        assert_eq!(variant_name(&err), "Array");
        let rendered = err.to_string();
        assert!(rendered.contains("first"), "{rendered}");
        assert!(rendered.contains("second"), "{rendered}");
    }
}
