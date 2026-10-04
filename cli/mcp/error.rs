//! Turning an [`oinkie::Error`] into something an MCP client can act on.
//!
//! Two things matter here. The message is carried through unchanged, because
//! the library's messages are the ones that name the canonical spelling of a
//! refused pairing or number the failures of a run over several inputs -- and
//! the reader is a model that has no `--help` to consult. And the choice
//! between "you asked for something impossible" and "something went wrong" is
//! the library's [`oinkie::Error::is_caller_fault`], made there by an
//! exhaustive match: `Error` is `#[non_exhaustive]`, so a match here would need
//! a wildcard arm, and a variant added later would fall into it unclassified.

use oinkie::Error;
use rmcp::ErrorData;

pub fn to_mcp(e: Error) -> ErrorData {
    let message = e.to_string();
    if e.is_caller_fault() {
        ErrorData::invalid_params(message, None)
    } else {
        ErrorData::internal_error(message, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The message is the library's, whichever side the fault falls on. It is
    /// the only thing the reader gets, and for a refused pairing it is the
    /// only place the canonical spelling appears.
    #[test]
    fn test_the_library_message_is_carried_through_unchanged() {
        let e = Error::IncompatibleAnalysis(
            oinkie::birthmarks::BirthmarkType::OpSeq,
            oinkie::compare::Algorithm::Euclidean,
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
    }
}
