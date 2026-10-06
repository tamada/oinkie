//! The Hex-Rays microcode.
//!
//! IDA Pro's decompiler does not translate assembly into C in one step. It
//! builds microcode and rewrites it through a pipeline, and `gen_microcode`
//! takes the stage to stop at. `MMAT_LVARS` is the last of them -- local
//! variables allocated, and what the pseudocode is rendered from -- and is
//! what oinkie reads.
//!
//! # Why only the last one
//!
//! Binary Ninja's LLIL, MLIL and HLIL are three representations it publishes,
//! and offering all three is offering a choice its users are expected to make.
//! The maturities are not that. They are where a plugin may intervene in the
//! decompiler's own pipeline, which the enum shows plainly: `MMAT_ZERO` is in
//! it, described as "microcode does not exist", and `MMAT_CALLS` is documented
//! by pointing at an event hook. `MMAT_GLBOPT2` Hex-Rays describes only as
//! "most global optimization passes are done".
//!
//! Nothing states that the intermediate shapes are stable across releases, and
//! a representation named here is permanent: a file carrying the name has to
//! stay readable. Measured on one binary, four of the eight produced the same
//! operations anyway -- `MMAT_GLBOPT3` is where Hex-Rays says the microcode is
//! fixed, so `MMAT_LVARS` differs from it only in what the variables are
//! called.
//!
//! Adding a maturity later costs nothing; removing one breaks every file that
//! named it.
//!
//! The opcode is a `String` for the reason [`crate::binaryninja`] gives at
//! length: the vocabulary belongs to the tool, arrives from its API, and grows
//! with releases, so a closed enum would refuse files a newer IDA wrote
//! correctly.

use serde::{Deserialize, Serialize};

pub(crate) mod cloud;
pub(crate) mod lifter;

/// The operations that transfer control to another function.
///
/// The microcode spells a call two ways and both are calls: `m_call` is
/// direct, `m_icall` indirect. Getting this wrong is the failure the whole
/// `fc-*` family rests on -- an `is_call` matching nothing leaves every such
/// birthmark empty, and two empty birthmarks score as a perfect match.
const CALL_OPS: &[&str] = &["m_call", "m_icall"];

/// One microcode instruction.
#[derive(Serialize, Deserialize, Debug)]
pub struct Op {
    op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    out: Option<String>,
    inputs: Vec<String>,
}

impl crate::Op for Op {
    fn mnemonic(&self) -> &str {
        &self.op
    }

    fn inputs(&self) -> &[String] {
        &self.inputs
    }

    fn ret(&self) -> Option<&str> {
        self.out.as_deref()
    }

    fn is_call(&self) -> bool {
        CALL_OPS.contains(&self.op.as_str())
    }

    fn symbol_key(&self) -> Option<String> {
        if !self.is_call() {
            return None;
        }
        // The script writes the callee as the first operand and keys the
        // symbol table the same way, so this hands back what it wrote. An
        // indirect call renders a register, which is in no table; looking it
        // up finds nothing, which is the same answer as returning None and is
        // reached without this deciding what a callee looks like.
        self.inputs.first().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Op as _;

    fn op(name: &str, inputs: &[&str]) -> Op {
        Op {
            op: name.to_string(),
            out: None,
            inputs: inputs.iter().map(ToString::to_string).collect(),
        }
    }

    /// Both spellings of a call are calls. `m_icall` is the one worth
    /// asserting: an indirect call is still a call, and leaving it out would
    /// quietly shrink every `fc-*` birthmark of a program that makes them.
    #[test]
    fn test_both_spellings_of_a_call_are_calls() {
        let direct = op("m_call", &["$_printf"]);
        let indirect = op("m_icall", &["r0"]);
        assert!(direct.is_call());
        assert!(indirect.is_call());
        assert_eq!(direct.symbol_key().as_deref(), Some("$_printf"));
    }

    #[test]
    fn test_an_operation_that_is_not_a_call_names_no_symbol() {
        let mov = op("m_mov", &["r0", "$_printf"]);
        assert!(!mov.is_call());
        assert_eq!(mov.symbol_key(), None);
    }

    #[test]
    fn test_a_call_without_operands_yields_nothing() {
        let bare = op("m_call", &[]);
        assert!(bare.is_call());
        assert_eq!(bare.symbol_key(), None);
    }

    /// The operands and the destination are handed back as the script wrote
    /// them.
    #[test]
    fn test_the_operands_and_the_destination_are_what_was_read() {
        let mut mov = op("m_mov", &["r1", "r0"]);
        assert_eq!(mov.inputs(), ["r1".to_string(), "r0".to_string()]);
        assert_eq!(mov.ret(), None);
        mov.out = Some("r0".to_string());
        assert_eq!(mov.ret(), Some("r0"));
        assert_eq!(mov.mnemonic(), "m_mov");
    }
}
