//! Binary Ninja's three intermediate languages.
//!
//! LLIL, MLIL and HLIL differ in vocabulary and in how they render a call,
//! and in nothing else this crate cares about, so one [`Op`] is written once
//! and parameterised by a [`Level`].
//!
//! # Why the opcode is a string here and an enum for Ghidra
//!
//! [`crate::ghidra::pcode::PcodeOp`] is a closed enum, and refusing an opcode
//! it does not know is a feature there: P-Code's operation set is defined by
//! Ghidra's model and changes rarely.
//!
//! Binary Ninja's does not behave that way. Each level has on the order of a
//! hundred operations, the three sets are disjoint, and new ones arrive with
//! new releases -- a closed enum would make this build refuse files that a
//! newer Binary Ninja produced correctly. Since the names come straight from
//! the tool's own API rather than from anything oinkie parses, the enum would
//! be guarding against a mistake nobody can make.
//!
//! What the enum *would* have protected is [`crate::Op::is_call`]: a
//! misspelled opcode there matches nothing, the `fc-*` birthmarks come out
//! empty, and two empty birthmarks score as a perfect match, so unrelated
//! programs are reported as identical. That is guarded instead by
//! `tests/binaryninja_test.rs`, which extracts `fc-*` from committed fixtures
//! and fails if the result is empty -- a check the compiler could not have
//! made anyway, since it cannot know what Binary Ninja emits.

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

pub(crate) mod lifter;

/// The word the lifting script takes for a representation, or `None` for one
/// that is not Binary Ninja's.
///
/// Exhaustive over [`crate::lift::Ir`] on purpose, rather than an
/// `unreachable!` arm for the representations that cannot reach it -- a
/// branch nothing can exercise. Adding a representation anywhere is a build
/// failure here, and a Binary Ninja level forgotten here is a `None` that
/// turns into a refusal.
pub(crate) fn level(ir: crate::lift::Ir) -> Option<&'static str> {
    use crate::lift::Ir;
    match ir {
        Ir::BinaryNinjaLlil => Some("llil"),
        Ir::BinaryNinjaMlil => Some("mlil"),
        Ir::BinaryNinjaHlil => Some("hlil"),
        Ir::GhidraPcode | Ir::IdaMicrocode => None,
    }
}

/// One of Binary Ninja's intermediate languages, as far as reading its
/// operations requires.
pub trait Level {
    /// The operations that transfer control to another function.
    ///
    /// Kept beside the lifting script's own list, which has to agree; the
    /// fixtures are what say they do.
    const CALL_OPS: &[&str];

    /// Which rendered operand of a call names the callee.
    ///
    /// LLIL puts the target first. MLIL puts its output variables first and
    /// the target second. HLIL renders the callee itself first.
    const CALL_TARGET: usize;
}

/// Low Level IL: one expression per machine instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Llil;

/// Medium Level IL: registers and stack slots resolved into variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mlil;

/// High Level IL: control flow recovered, symbols already resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hlil;

impl Level for Llil {
    const CALL_OPS: &[&str] = &[
        "LLIL_CALL",
        "LLIL_CALL_STACK_ADJUST",
        "LLIL_TAILCALL",
        "LLIL_SYSCALL",
    ];
    const CALL_TARGET: usize = 0;
}

impl Level for Mlil {
    const CALL_OPS: &[&str] = &[
        "MLIL_CALL",
        "MLIL_CALL_UNTYPED",
        "MLIL_TAILCALL",
        "MLIL_TAILCALL_UNTYPED",
        "MLIL_SYSCALL",
        "MLIL_SYSCALL_UNTYPED",
    ];
    const CALL_TARGET: usize = 1;
}

impl Level for Hlil {
    const CALL_OPS: &[&str] = &["HLIL_CALL", "HLIL_TAILCALL", "HLIL_SYSCALL"];
    const CALL_TARGET: usize = 0;
}

/// One instruction of a Binary Ninja intermediate language.
#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub struct Op<L> {
    op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    out: Option<String>,
    inputs: Vec<String>,
    #[serde(skip)]
    level: PhantomData<L>,
}

impl<L: Level> crate::Op for Op<L> {
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
        L::CALL_OPS.contains(&self.op.as_str())
    }

    fn symbol_key(&self) -> Option<String> {
        if !self.is_call() {
            return None;
        }
        // The key is the operand as this level renders it, which is what the
        // lifting script keyed the symbol table by: an address at LLIL and
        // MLIL, the name itself at HLIL. An indirect call renders a register
        // or a variable, which is in no table, and looking it up finds
        // nothing -- the same answer as returning None, reached without this
        // deciding what a callee looks like at each level.
        self.inputs.get(L::CALL_TARGET).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Op as _;

    fn op<L>(name: &str, inputs: &[&str]) -> Op<L> {
        Op {
            op: name.to_string(),
            out: None,
            inputs: inputs.iter().map(ToString::to_string).collect(),
            level: PhantomData,
        }
    }

    /// Each level reads the callee from its own operand position, which is
    /// the only thing that differs between them.
    #[test]
    fn test_each_level_reads_the_callee_from_its_own_position() {
        let llil: Op<Llil> = op("LLIL_CALL", &["0x100000480"]);
        assert_eq!(llil.symbol_key().as_deref(), Some("0x100000480"));

        let mlil: Op<Mlil> = op("MLIL_CALL", &["[]", "0x100000480", "[]"]);
        assert_eq!(mlil.symbol_key().as_deref(), Some("0x100000480"));

        let hlil: Op<Hlil> = op("HLIL_CALL", &["_printf", "[]"]);
        assert_eq!(hlil.symbol_key().as_deref(), Some("_printf"));
    }

    /// The operands and the destination are handed back as the script wrote
    /// them. `ret` is `None` for most operations because Binary Ninja reports
    /// no variable written, and the op-* families read `inputs` rather than
    /// the mnemonic alone.
    #[test]
    fn test_the_operands_and_the_destination_are_what_was_read() {
        let mut set: Op<Llil> = op("LLIL_SET_REG", &["x0", "0x100000000"]);
        assert_eq!(set.inputs(), ["x0".to_string(), "0x100000000".to_string()]);
        assert_eq!(set.ret(), None);

        set.out = Some("x0".to_string());
        assert_eq!(set.ret(), Some("x0"));

        let empty: Op<Hlil> = op("HLIL_NOP", &[]);
        assert!(empty.inputs().is_empty());
    }

    /// Only Binary Ninja's representations name a level, and each names its
    /// own. The `None` half is what lets the lifter refuse rather than panic.
    #[test]
    fn test_only_binary_ninjas_representations_name_a_level() {
        use crate::lift::Ir;
        assert_eq!(level(Ir::BinaryNinjaLlil), Some("llil"));
        assert_eq!(level(Ir::BinaryNinjaMlil), Some("mlil"));
        assert_eq!(level(Ir::BinaryNinjaHlil), Some("hlil"));
        assert_eq!(level(Ir::GhidraPcode), None);
        assert_eq!(level(Ir::IdaMicrocode), None);
    }

    /// An operation that is not a call names no symbol, however its operands
    /// happen to be shaped.
    #[test]
    fn test_only_a_call_names_a_symbol() {
        let set: Op<Llil> = op("LLIL_SET_REG", &["x0", "0x100000480"]);
        assert!(!set.is_call());
        assert_eq!(set.symbol_key(), None);
    }

    /// A call with no operands at all yields nothing rather than panicking.
    #[test]
    fn test_a_call_without_its_target_operand_yields_nothing() {
        let truncated: Op<Mlil> = op("MLIL_CALL", &["[]"]);
        assert!(truncated.is_call());
        assert_eq!(truncated.symbol_key(), None);
    }

    /// The three sets are disjoint, so a level cannot accidentally recognise
    /// another's call and report a birthmark built from the wrong vocabulary.
    #[test]
    fn test_no_level_recognises_another_levels_call() {
        let llil_call_at_mlil: Op<Mlil> = op("LLIL_CALL", &["0x1"]);
        assert!(!llil_call_at_mlil.is_call());
        let hlil_call_at_llil: Op<Llil> = op("HLIL_CALL", &["_printf"]);
        assert!(!hlil_call_at_llil.is_call());
    }
}
