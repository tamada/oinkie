//! Turning an [`oinkie::Error`] into something an MCP client can act on.
//!
//! Two things matter here. The message is carried through unchanged, because
//! the library's messages are the ones that name the canonical spelling of a
//! refused pairing or number the failures of a run over several inputs -- and
//! the reader is a model that has no `--help` to consult. And the choice
//! between "you asked for something impossible" and "something went wrong" is
//! made by an exhaustive match, so a variant added later stops the build
//! rather than defaulting to whichever is wrong for it.
//!
//! Written here rather than alongside the first tool that needs it. Nothing in
//! this change can fail -- `oinkie_info` takes no arguments and reads no files
//! -- but the match below has to decide, once, which side of the line every
//! existing variant falls on, and doing that deliberately with tests is worth
//! more than doing it in a hurry next to the tools that will use it.
#![allow(
    dead_code,
    reason = "used by the tools that take arguments and read files"
)]

use oinkie::Error;
use rmcp::ErrorData;

/// Whether the caller could have avoided this by asking differently.
fn is_the_callers_fault(e: &Error) -> bool {
    match e {
        // A name, a pairing or a number that the caller supplied.
        Error::BirthmarkType(_)
        | Error::IncompatibleAnalysis(_, _)
        | Error::Mismatch(_, _)
        | Error::IrMismatch(_, _)
        | Error::Parse(_)
        | Error::ParseFloat(_, _)
        | Error::ParseInt(_, _)
        | Error::Clap(_) => true,

        // A file the caller named, which they can name differently.
        Error::Io(_, _) | Error::Json(_, _) | Error::UnsupportedIr(_, _) => true,

        // The program really has no calls, so no fc-* birthmark of it exists.
        // The caller can ask for a different birthmark, so it is theirs.
        Error::NoCallOperations(_, _) => true,

        // Something went wrong inside, or in a file oinkie itself produced.
        Error::Csv(_)
        | Error::InvalidPcode(_)
        | Error::LapJV(_)
        | Error::ShapeError(_)
        | Error::UnreadableOutput(_, _) => false,

        // A group is the caller's fault only if all of it is. One internal
        // failure in a batch makes the whole batch an internal failure, since
        // saying "you asked wrongly" about it would be wrong for that one.
        Error::Array(errs) => errs.iter().all(is_the_callers_fault),
    }
}

pub fn to_mcp(e: Error) -> ErrorData {
    let message = e.to_string();
    if is_the_callers_fault(&e) {
        ErrorData::invalid_params(message, None)
    } else {
        ErrorData::internal_error(message, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The message is the library's, whichever side the fault falls on. It is
    /// the only thing the reader gets, and for a refused pairing it is the
    /// only place the canonical spelling appears.
    #[test]
    fn test_the_library_message_is_carried_through_unchanged() {
        let e = Error::IncompatibleAnalysis(
            oinkie::prelude::BirthmarkType::OpSeq,
            oinkie::prelude::Algorithm::Euclidean,
        );
        let expected = e.to_string();
        assert!(expected.contains("op-freq-euclidean"), "{expected}");
        assert_eq!(to_mcp(e).message, expected);
    }

    #[test]
    fn test_a_bad_name_is_the_callers_fault() {
        let e = Error::BirthmarkType("nonsense".to_string());
        assert_eq!(
            to_mcp(e).code,
            rmcp::model::ErrorCode::INVALID_PARAMS,
            "an unknown birthmark name is something the caller can fix"
        );
    }

    #[test]
    fn test_an_internal_failure_is_not_blamed_on_the_caller() {
        let e = Error::InvalidPcode(9999);
        assert_ne!(to_mcp(e).code, rmcp::model::ErrorCode::INVALID_PARAMS);
    }

    /// The numbering survives, because "the third input failed" is the only
    /// thing a caller running over several files can act on.
    #[test]
    fn test_a_group_keeps_its_numbering() {
        let e = Error::Array(vec![
            Error::Parse("first".to_string()),
            Error::Parse("second".to_string()),
        ]);
        let d = to_mcp(e);
        assert!(d.message.contains("1. Parse error: first"), "{}", d.message);
        assert!(
            d.message.contains("2. Parse error: second"),
            "{}",
            d.message
        );
        assert_eq!(d.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    }

    /// One internal failure makes the batch internal. Reporting "you asked
    /// wrongly" for a group containing something the caller could not have
    /// avoided would be wrong about that one.
    #[test]
    fn test_one_internal_failure_makes_the_whole_group_internal() {
        let e = Error::Array(vec![
            Error::Parse("the caller's".to_string()),
            Error::UnreadableOutput(
                PathBuf::from("bin/sample"),
                Box::new(Error::Parse("ours".to_string())),
            ),
        ]);
        assert_ne!(to_mcp(e).code, rmcp::model::ErrorCode::INVALID_PARAMS);
    }
}
