//! The two `--` options whose values are names from the birthmark vocabulary.
//!
//! Both used to be `ValueEnum`s that hand-listed what the library already
//! parses: `BType` with 31 variants, `Analysis` with 64, at two different
//! k ceilings and in two spellings that were each different from the
//! library's (#25). The parsing is the library's now. What is kept here is
//! the part clap genuinely needs and the library cannot supply: a value
//! parser, and a finite list of names for completion and `--help`.
//!
//! clap uses `possible_values` for help, completion and its "did you mean"
//! suggestions, and leaves validation to `parse_ref`. That is what lets the
//! advertised list stop at `MAX_ADVERTISED_K` while `--analysis
//! op-9gram-set-dice` still runs.

use clap::builder::{PossibleValue, TypedValueParser};
use clap::error::ErrorKind;
use oinkie::prelude::{AnalysisType, BirthmarkType};
use std::ffi::OsStr;

/// The `--analysis` value, kept as the name it was given.
///
/// The name rather than the parsed `AnalysisType`, because that owns a
/// `Comparator` and clap needs the option's type to be `Clone`. It is parsed
/// once for validation when the argument is read, so holding the name is not
/// holding something unchecked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Analysis(String);

impl Analysis {
    pub fn analysis_type(&self) -> oinkie::Result<AnalysisType> {
        AnalysisType::try_from(self.0.as_str())
    }
}

#[derive(Clone)]
pub struct AnalysisParser;

impl TypedValueParser for AnalysisParser {
    type Value = Analysis;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let name = lossless(value, cmd, "analysis name")?;
        // Parsed and thrown away: this is the validation, and doing it here is
        // what keeps an unknown name a clap error rather than a failure after
        // the run has started. The message is the library's, which names the
        // canonical pairing when the algorithm does not match the shape.
        AnalysisType::try_from(name.as_str()).map_err(|e| invalid_value(cmd, e))?;
        Ok(Analysis(name))
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            AnalysisType::advertised_names().map(PossibleValue::new),
        ))
    }
}

#[derive(Clone)]
pub struct BirthmarkTypeParser;

impl TypedValueParser for BirthmarkTypeParser {
    type Value = BirthmarkType;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let name = lossless(value, cmd, "birthmark type")?;
        BirthmarkType::try_from(name.as_str()).map_err(|e| invalid_value(cmd, e))
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(BirthmarkType::advertised().map(|bt| {
            PossibleValue::new(bt.to_string()).help(bt.description())
        })))
    }
}

/// A name that is not UTF-8 cannot be one of these, so it is rejected as an
/// invalid value rather than lossily converted into a different name that
/// might happen to parse.
///
/// `what` names the vocabulary the value failed to be. Both parsers come
/// through here, and an `--analysis` value told it is not a birthmark name
/// sends the reader looking in the wrong place.
fn lossless(value: &OsStr, cmd: &clap::Command, what: &str) -> Result<String, clap::Error> {
    value.to_str().map(str::to_string).ok_or_else(|| {
        clap::Error::raw(
            ErrorKind::InvalidUtf8,
            format!("{:?}: not a valid {what}\n", value),
        )
        .with_cmd(cmd)
    })
}

fn invalid_value(cmd: &clap::Command, e: oinkie::Error) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, format!("{e}\n")).with_cmd(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::builder::TypedValueParser;

    fn advertised<P: TypedValueParser>(parser: &P) -> Vec<PossibleValue> {
        parser
            .possible_values()
            .expect("the option offers a list")
            .collect()
    }

    /// What clap advertises is what the library generated, not a copy of it.
    /// The two hand-written lists this replaced went stale in three
    /// directions at once (#25), so the assertion is that nothing is
    /// restated here.
    #[test]
    fn test_the_offered_analyses_are_the_ones_the_library_generates() {
        let offered = advertised(&AnalysisParser)
            .iter()
            .map(|pv| pv.get_name().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            offered,
            AnalysisType::advertised_names().collect::<Vec<_>>()
        );
        assert!(offered.contains(&"op-3gram-set-dice".to_string()));
    }

    #[test]
    fn test_the_offered_birthmarks_are_the_ones_the_library_generates() {
        let offered = advertised(&BirthmarkTypeParser);
        let expected = BirthmarkType::advertised().collect::<Vec<_>>();
        assert_eq!(offered.len(), expected.len());
        for (pv, bt) in offered.iter().zip(expected) {
            assert_eq!(pv.get_name(), bt.to_string());
            // the help is what `--help` and the shells show beside the name
            assert_eq!(pv.get_help().map(|h| h.to_string()), Some(bt.description()));
        }
    }

    /// Both options take a name the library parses, and the list is only
    /// what gets suggested: a k past the end of it is still accepted.
    #[test]
    fn test_a_name_past_the_end_of_the_list_still_parses() {
        let cmd = clap::Command::new("oinkie");
        let name = format!("op-{}gram-set-dice", oinkie::prelude::MAX_ADVERTISED_K + 1);
        assert!(
            !advertised(&AnalysisParser)
                .iter()
                .any(|pv| pv.get_name() == name)
        );
        assert!(
            AnalysisParser
                .parse_ref(&cmd, None, OsStr::new(name.as_str()))
                .is_ok()
        );
    }

    #[test]
    fn test_a_name_the_library_refuses_is_refused_here() {
        let cmd = clap::Command::new("oinkie");
        for (name, expected) in [
            ("nonsense", "unknown birthmark type"),
            ("op-seq-euclidean", "use op-freq-euclidean"),
        ] {
            let Err(e) = AnalysisParser.parse_ref(&cmd, None, OsStr::new(name)) else {
                panic!("{name} should be refused");
            };
            assert!(e.to_string().contains(expected), "{name}: {e}");
        }
        let Err(e) = BirthmarkTypeParser.parse_ref(&cmd, None, OsStr::new("op-tri-gram-set"))
        else {
            panic!("the old spelling should be refused");
        };
        assert!(e.to_string().contains("unknown birthmark type"), "{e}");
    }

    #[test]
    fn test_a_name_the_library_accepts_comes_back_parsed() {
        let cmd = clap::Command::new("oinkie");
        let a = AnalysisParser
            .parse_ref(&cmd, None, OsStr::new("op-3gram-set-dice"))
            .unwrap();
        assert_eq!(
            a.analysis_type().unwrap().birthmark,
            BirthmarkType::OpKgramSet(3)
        );
        let bt = BirthmarkTypeParser
            .parse_ref(&cmd, None, OsStr::new("op-3gram-set"))
            .unwrap();
        assert_eq!(bt, BirthmarkType::OpKgramSet(3));
    }

    /// Both parsers refuse a non-UTF-8 value through the same helper, so the
    /// message has to name the vocabulary the value failed to be. It said
    /// "birthmark name" for both until a review caught it, which sent an
    /// `--analysis` reader looking in the wrong place.
    #[cfg(unix)]
    #[test]
    fn test_a_non_utf8_value_is_refused_as_what_the_option_takes() {
        use std::os::unix::ffi::OsStrExt;
        let cmd = clap::Command::new("oinkie");
        let bad = OsStr::from_bytes(b"op-\xff-set");

        let e = AnalysisParser
            .parse_ref(&cmd, None, bad)
            .expect_err("a non-UTF-8 value cannot be a name");
        assert!(e.to_string().contains("analysis name"), "{e}");

        let e = BirthmarkTypeParser
            .parse_ref(&cmd, None, bad)
            .expect_err("a non-UTF-8 value cannot be a name");
        assert!(e.to_string().contains("birthmark type"), "{e}");
    }
}
