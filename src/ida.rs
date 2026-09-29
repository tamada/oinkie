//! The Hex-Rays microcode, at each of its maturities.
//!
//! IDA Pro's decompiler does not produce one representation. It rewrites the
//! microcode through a pipeline, and the vocabulary at each stage is its own:
//! `MMAT_GENERATED` still holds the instructions as lifted, while
//! `MMAT_LVARS`, the last, is what the pseudocode is rendered from. Which one
//! answers a question is the caller's, so all of them are offered.
//!
//! `MMAT_ZERO` is not among them. It is the state before any microcode has
//! been generated, so there is nothing to write -- an absence rather than a
//! representation.
//!
//! The opcode is a `String` for the reason [`crate::binaryninja`] gives at
//! length: the vocabulary belongs to the tool, arrives from its API, and grows
//! with releases, so a closed enum would refuse files a newer IDA wrote
//! correctly.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

pub(crate) mod cloud;
pub(crate) mod lifter;

/// The word the lifting script takes for a representation, or `None` for one
/// that is not IDA's.
///
/// Exhaustive over [`crate::lift::Ir`], for the reason
/// [`crate::binaryninja::level`] is: a match with an unreachable arm is a
/// branch nothing can check, and adding a maturity and forgetting it here
/// becomes a refusal rather than a panic.
pub(crate) fn maturity(ir: crate::lift::Ir) -> Option<&'static str> {
    use crate::lift::Ir;
    Some(match ir {
        Ir::IdaMicrocodeGenerated => Generated::NAME,
        Ir::IdaMicrocodePreoptimized => Preoptimized::NAME,
        Ir::IdaMicrocodeLocopt => Locopt::NAME,
        Ir::IdaMicrocodeCalls => Calls::NAME,
        Ir::IdaMicrocodeGlbopt1 => Glbopt1::NAME,
        Ir::IdaMicrocodeGlbopt2 => Glbopt2::NAME,
        Ir::IdaMicrocodeGlbopt3 => Glbopt3::NAME,
        Ir::IdaMicrocodeLvars => Lvars::NAME,
        Ir::GhidraPcode | Ir::BinaryNinjaLlil | Ir::BinaryNinjaMlil | Ir::BinaryNinjaHlil => {
            return None;
        }
    })
}

/// The operations that transfer control to another function.
///
/// The microcode spells a call two ways and both are calls: `m_call` is
/// direct, `m_icall` indirect. Getting this wrong is the failure the whole
/// `fc-*` family rests on -- an `is_call` matching nothing leaves every such
/// birthmark empty, and two empty birthmarks score as a perfect match.
const CALL_OPS: &[&str] = &["m_call", "m_icall"];

/// A maturity of the microcode, as far as reading its operations requires.
///
/// The maturities differ in vocabulary rather than in how a call is written,
/// so unlike Binary Ninja's levels there is nothing here to vary yet. It
/// exists so that the eight are distinct types -- which is what lets
/// `AnyProgram` hold them apart and `compare_any` refuse to mix them.
pub trait Maturity {
    /// The word the lifting script takes, and the tail of this
    /// representation's name.
    const NAME: &'static str;
}

macro_rules! maturity {
    ($($t:ident => $n:literal,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $t;
            impl Maturity for $t {
                const NAME: &'static str = $n;
            }
        )*
    };
}

maturity! {
    Generated => "generated",
    Preoptimized => "preoptimized",
    Locopt => "locopt",
    Calls => "calls",
    Glbopt1 => "glbopt1",
    Glbopt2 => "glbopt2",
    Glbopt3 => "glbopt3",
    Lvars => "lvars",
}

/// One microcode instruction.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub struct Op<M> {
    op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    out: Option<String>,
    inputs: Vec<String>,
    #[serde(skip)]
    maturity: PhantomData<M>,
}

impl<M: Maturity> crate::Op for Op<M> {
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

    fn op<M>(name: &str, inputs: &[&str]) -> Op<M> {
        Op {
            op: name.to_string(),
            out: None,
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            maturity: PhantomData,
        }
    }

    /// Both spellings of a call are calls. `m_icall` was the one worth
    /// asserting: an indirect call is still a call, and leaving it out would
    /// quietly shrink every `fc-*` birthmark of a program that makes them.
    #[test]
    fn test_both_spellings_of_a_call_are_calls() {
        let direct: Op<Lvars> = op("m_call", &["_printf"]);
        let indirect: Op<Lvars> = op("m_icall", &["r0"]);
        assert!(direct.is_call());
        assert!(indirect.is_call());
        assert_eq!(direct.symbol_key().as_deref(), Some("_printf"));
    }

    #[test]
    fn test_an_operation_that_is_not_a_call_names_no_symbol() {
        let mov: Op<Generated> = op("m_mov", &["r0", "_printf"]);
        assert!(!mov.is_call());
        assert_eq!(mov.symbol_key(), None);
    }

    #[test]
    fn test_a_call_without_operands_yields_nothing() {
        let bare: Op<Glbopt1> = op("m_call", &[]);
        assert!(bare.is_call());
        assert_eq!(bare.symbol_key(), None);
    }

    /// The operands and the destination are handed back as the script wrote
    /// them.
    #[test]
    fn test_the_operands_and_the_destination_are_what_was_read() {
        let mut mov: Op<Calls> = op("m_mov", &["r1", "r0"]);
        assert_eq!(mov.inputs(), ["r1".to_string(), "r0".to_string()]);
        assert_eq!(mov.ret(), None);
        mov.out = Some("r0".to_string());
        assert_eq!(mov.ret(), Some("r0"));
        assert_eq!(mov.mnemonic(), "m_mov");
    }

    /// Every maturity names itself, and no two name the same thing -- the
    /// names become part of the file format through `Ir`.
    #[test]
    fn test_the_maturities_have_distinct_names() {
        let names = [
            Generated::NAME,
            Preoptimized::NAME,
            Locopt::NAME,
            Calls::NAME,
            Glbopt1::NAME,
            Glbopt2::NAME,
            Glbopt3::NAME,
            Lvars::NAME,
        ];
        let mut sorted = names.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "two maturities share a name");
        assert!(!names.iter().any(|n| n.is_empty()));
    }
}
