use std::path::PathBuf;

use ndarray::ShapeError;

use crate::prelude::BirthmarkType;

mod birthmarks;
mod compare;
pub mod extractor;
pub mod ghidra;
pub mod lift;
pub mod prelude;
mod program;

pub type Result<T> = std::result::Result<T, Error>;

/// The messages live on the variants rather than in a `Display` match, so
/// that adding a variant and deciding how it reads are the same edit.
///
/// The wrapped errors are `#[source]` but not `#[from]`. `Io` and `Json`
/// carry the path beside the error — which path failed is the useful half —
/// so they could not be `#[from]` anyway, and for the rest an explicit
/// `map_err(Error::Csv)` at the call site says more than a conversion hidden
/// inside a `?`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", render_group(.0))]
    Array(Vec<Self>),
    #[error("{0}: unknown birthmark type")]
    BirthmarkType(String),
    /// clap's own message already begins `error: `, so this adds no prefix of
    /// its own. It used to read `Clap error: error: ...` (#62).
    #[error("{0}")]
    Clap(#[source] clap::Error),
    #[error("CSV error: {0}")]
    Csv(#[source] csv::Error),
    /// A birthmark shape paired with an algorithm that does not operate on it.
    #[error("{}", render_incompatible(.0, .1))]
    IncompatibleAnalysis(BirthmarkType, crate::prelude::Algorithm),
    /// Two birthmarks lifted to different intermediate representations.
    #[error(
        "cannot compare {0} against {1}: the two are lifted to different intermediate representations, whose operation vocabularies do not correspond"
    )]
    IrMismatch(crate::lift::Ir, crate::lift::Ir),
    /// An `fc-*` birthmark was asked of a program holding no call at all.
    #[error(
        "{path}: no operation is a call, so every fc-* birthmark of it would be empty -- and two empty birthmarks score as a perfect match. Either the program really calls nothing, or oinkie's reader for {ir} does not recognise that representation's call operations",
        path = .0.display(),
        ir = .1
    )]
    NoCallOperations(PathBuf, crate::lift::Ir),
    /// A lifted file naming a representation this build cannot read.
    #[error(
        "{path}: no reader for {ir}; this build can read {readable}",
        path = .0.display(),
        ir = .1,
        readable = render_readable()
    )]
    UnsupportedIr(PathBuf, crate::lift::Ir),
    #[error("invalid pcode: {0}")]
    InvalidPcode(u32),
    #[error("IO error for {path}: {cause}", path = .0.display(), cause = .1)]
    Io(PathBuf, #[source] std::io::Error),
    #[error("{path}: JSON error: {cause}", path = .0.display(), cause = .1)]
    Json(PathBuf, #[source] serde_json::Error),
    #[error("LapJV error: {0}")]
    LapJV(#[source] lapjv::LapJVError),
    #[error("Mismatched birthmark types: {0} and {1}")]
    Mismatch(BirthmarkType, BirthmarkType),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("{0}: Parse float error {1}")]
    ParseFloat(String, #[source] std::num::ParseFloatError),
    #[error("{0}: Parse int error {1}")]
    ParseInt(String, #[source] std::num::ParseIntError),
    #[error("Shape error: {0}")]
    ShapeError(#[source] ShapeError),
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

fn render_incompatible(bt: &BirthmarkType, algorithm: &crate::prelude::Algorithm) -> String {
    let name = algorithm.cli_name();
    format!(
        "{bt}-{name}: {name} operates on {}; use {}-{name}",
        algorithm.shape().description(),
        bt.with_shape(algorithm.shape())
    )
}

fn render_readable() -> String {
    crate::lift::Ir::readable()
        .iter()
        .map(|ir| ir.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl Error {
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

    pub fn error_or<T>(result: T, errs: Vec<Self>) -> Result<T> {
        if errs.is_empty() {
            Ok(result)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Self::Array(errs))
        }
    }
}

pub trait Op {
    /// returns the mnemonic of the operation, e.g., "ADD", "SUB", etc.
    fn mnemonic(&self) -> &str;

    /// returns the inputs of the operation, e.g., the source registers or memory locations.
    fn inputs(&self) -> &[String];

    /// returns whether this operation transfers control to another function,
    /// which is what the `fc-*` birthmarks are built from.
    ///
    /// Each intermediate representation spells its call differently — P-Code
    /// writes `CALL`, the Hex-Rays microcode `m_call`, Binary Ninja's LLIL
    /// `LLIL_CALL` — and some name more than one. Deciding here rather than in
    /// [`crate::extractor`] keeps that vocabulary with the lifter that owns
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
    fn ret(&self) -> Option<&str>;

    /// returns this operation's first operand rendered as the program's symbol
    /// table keys it, so that [`crate::program::Program::symbol`] can resolve
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

pub trait Iterable {
    type Item;
    fn iter(&self) -> Box<dyn Iterator<Item = &Self::Item> + '_>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lift::Ir;
    use crate::prelude::Algorithm;

    /// Adding a variant makes this stop compiling, which is the reminder to
    /// add it to `rendered_errors` below as well. There is no way to
    /// enumerate an enum's variants, so an exhaustive match is the closest a
    /// test can get to noticing that it has fallen behind.
    fn variant_name(e: &Error) -> &'static str {
        match e {
            Error::Array(_) => "Array",
            Error::BirthmarkType(_) => "BirthmarkType",
            Error::Clap(_) => "Clap",
            Error::Csv(_) => "Csv",
            Error::IncompatibleAnalysis(_, _) => "IncompatibleAnalysis",
            Error::IrMismatch(_, _) => "IrMismatch",
            Error::NoCallOperations(_, _) => "NoCallOperations",
            Error::UnsupportedIr(_, _) => "UnsupportedIr",
            Error::InvalidPcode(_) => "InvalidPcode",
            Error::Io(_, _) => "Io",
            Error::Json(_, _) => "Json",
            Error::LapJV(_) => "LapJV",
            Error::Mismatch(_, _) => "Mismatch",
            Error::Parse(_) => "Parse",
            Error::ParseFloat(_, _) => "ParseFloat",
            Error::ParseInt(_, _) => "ParseInt",
            Error::ShapeError(_) => "ShapeError",
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
        let clap_err = clap::Error::raw(clap::error::ErrorKind::InvalidValue, "boom");

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
        let (io_msg, clap_msg) = (io_err.to_string(), clap_err.to_string());

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
            (
                // No prefix of its own: clap's message already begins
                // "error: ", and this used to put "Clap error: " in front of
                // that (#62).
                Error::Clap(clap_err),
                clap_msg.clone(),
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
                Error::NoCallOperations(PathBuf::from("bin/sample"), Ir::GhidraPcode),
                "bin/sample: no operation is a call, so every fc-* birthmark of it would be empty -- and two empty birthmarks score as a perfect match. Either the program really calls nothing, or oinkie's reader for ghidra-pcode does not recognise that representation's call operations".to_string(),
            ),
            (
                Error::UnsupportedIr(PathBuf::from("foreign.json"), Ir::IdaMicrocode),
                "foreign.json: no reader for ida-microcode; this build can read ghidra-pcode"
                    .to_string(),
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
        ]
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

    /// The message has to name every representation this build can read, and
    /// separate them.
    ///
    /// Honest about its reach: `Ir::readable()` holds one entry today, so the
    /// separator half cannot fail yet — `join("")` and `join(", ")` produce
    /// the same string for one item. It is here for the build that adds a
    /// second reader, which is exactly when a broken join would ship
    /// unnoticed.
    #[test]
    fn test_an_unreadable_representation_lists_what_can_be_read() {
        let rendered =
            Error::UnsupportedIr(PathBuf::from("foreign.json"), crate::lift::Ir::IdaMicrocode)
                .to_string();
        for ir in crate::lift::Ir::readable() {
            assert!(
                rendered.contains(&ir.to_string()),
                "{ir} missing: {rendered}"
            );
        }
        let separators = rendered.matches(", ").count();
        assert_eq!(
            separators,
            crate::lift::Ir::readable().len() - 1,
            "{} readable representations should be separated {} times: {rendered}",
            crate::lift::Ir::readable().len(),
            crate::lift::Ir::readable().len() - 1
        );
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
