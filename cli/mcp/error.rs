//! Turning an [`oinkie::Error`] into something an MCP client can act on.
//!
//! Two things matter here. The message is carried through unchanged, because
//! the library's messages are the ones that name the canonical spelling of a
//! refused pairing or number the failures of a run over several inputs -- and
//! the reader is a model that has no `--help` to consult. And the choice
//! between "you asked for something impossible" and "something went wrong" is
//! made by an exhaustive match, so a variant added later stops the build
//! rather than defaulting to whichever is wrong for it.

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

        // `Parse` is a catch-all carrying a string, and the strings it carries
        // come from both sides: "Invalid aggregator" is the caller's, while
        // "Ghidra headless analyzer not found", "could not start N lift jobs"
        // and "angr is not yet implemented" are not. Nothing in the variant
        // says which.
        //
        // So it does not claim the caller is at fault. Telling a model it
        // asked wrongly when it did not is the more expensive mistake -- it
        // will try different arguments, repeatedly, against something no
        // argument can fix. A tool wanting a caller-fault code for a value it
        // took (the aggregator is what reaches `Parse` that way) should
        // validate that value itself rather than rely on this.
        Error::Parse(_) => false,

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

    /// `Parse` carries a string and nothing else, and those strings come from
    /// both sides. Since the variant cannot say which, it does not claim the
    /// caller is at fault -- no argument would fix this one.
    #[test]
    fn test_the_catch_all_does_not_claim_the_caller_is_at_fault() {
        let e = Error::Parse("Ghidra headless analyzer not found at /nope".to_string());
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
        assert_eq!(to_mcp(e).code, rmcp::model::ErrorCode::INVALID_PARAMS);
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
        assert_ne!(to_mcp(e).code, rmcp::model::ErrorCode::INVALID_PARAMS);
    }
}
