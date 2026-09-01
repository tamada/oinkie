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

impl std::fmt::Display for Analysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        let name = lossless(value, cmd)?;
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
        let name = lossless(value, cmd)?;
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
fn lossless(value: &OsStr, cmd: &clap::Command) -> Result<String, clap::Error> {
    value.to_str().map(str::to_string).ok_or_else(|| {
        clap::Error::raw(
            ErrorKind::InvalidUtf8,
            format!("{:?}: not a valid birthmark name\n", value),
        )
        .with_cmd(cmd)
    })
}

fn invalid_value(cmd: &clap::Command, e: oinkie::Error) -> clap::Error {
    clap::Error::raw(ErrorKind::InvalidValue, format!("{e}\n")).with_cmd(cmd)
}
