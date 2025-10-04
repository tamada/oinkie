use std::collections::{HashMap, HashSet};

use crate::birthmarks::Element;
use crate::Result;

pub(super) fn extract_freq(module: &llvm_ir::Module) -> Result<Vec<Element>> {
    let mut freq = HashMap::new();
    for func in &module.functions {
        for bb in &func.basic_blocks {
            for instr in &bb.instrs {
                let opcode = instruction_to_str(instr);
                *freq.entry(opcode).or_insert(0) += 1;
            }
            *freq.entry(terminator_to_str(&bb.term)).or_insert(0) += 1;
        }
    }
    Ok(freq.into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
}

pub(super) fn extract_set(module: &llvm_ir::Module) -> Result<Vec<Element>> {
    let mut opcodes = HashSet::new();
    for func in &module.functions {
        for bb in &func.basic_blocks {
            for instr in &bb.instrs {
                let opcode = instruction_to_str(instr);
                opcodes.insert(opcode.to_string());
            }
            opcodes.insert(terminator_to_str(&bb.term));
        }
    }
    Ok(opcodes.into_iter().map(|e| Element::Str(e)).collect::<Vec<_>>())
}

pub(super) fn extract_seq(module: &llvm_ir::Module) -> Result<Vec<Element>> {
    let mut opcodes = Vec::new();
    for func in &module.functions {
        for bb in &func.basic_blocks {
            for instr in &bb.instrs {
                let opcode = instruction_to_str(instr);
                opcodes.push(Element::Str(opcode.to_string()));
            }
            opcodes.push(Element::Str(terminator_to_str(&bb.term)));
        }
    }
    Ok(opcodes)
}

pub(super) fn extract_kgram(module: &llvm_ir::Module, k: usize) -> Result<Vec<Element>> {
    let mut kgrams = Vec::new();
    for func in &module.functions {
        let mut kgram = KGram::new(k);
        for bb in &func.basic_blocks {
            for inst in &bb.instrs {
                kgram.push(instruction_to_str(inst));
                if kgram.is_valid() {
                    kgrams.push(kgram.to_elem());
                }
            }
            kgram.push(terminator_to_str(&bb.term));
            if kgram.is_valid() {
                kgrams.push(kgram.to_elem())
            }
        }
    }
    Ok(kgrams)
}

#[derive(Clone)]
struct KGram {
    value: usize,
    elements: Vec<String>
}

impl KGram {
    pub fn new(k: usize) -> Self {
        KGram{ value: k, elements: vec![] }
    }
    pub fn is_valid(&self) -> bool {
        self.elements.len() == self.value
    }
    pub fn push(&mut self, e: String) {
        self.elements.push(e);
        if self.elements.len() > self.value {
            self.elements.remove(0);
        }
    }
    pub fn to_elem(&self) -> Element {
        Element::Kgram(self.elements.clone())
    }
}

fn terminator_to_str(term: &llvm_ir::Terminator) -> String {
    use llvm_ir::Terminator::*;
    match term {
        Ret(_ret) => "Ret",
        Br(_br) => "Br",
        CondBr(_cond_br) => "CondBr",
        Switch(_switch) => "Switch",
        IndirectBr(_indirect_br) => "IndirectBr",
        Invoke(_invoke) => "Invoke",
        Resume(_resume) => "Resume",
        Unreachable(_unreachable) => "Unreachable",
        CleanupRet(_cleanup_ret) => "CleanupRet",
        CatchRet(_catch_ret) => "CatchRet",
        CatchSwitch(_catch_switch) => "CatchSwitch",
        CallBr(_call_br) => "CallBr",
    }.to_string()
}

fn instruction_to_str(inst: &llvm_ir::Instruction) -> String {
    use llvm_ir::Instruction::*;
    match inst {
        Add(_item) => "Add",
        Sub(_item) => "Sub",
        Mul(_item) => "Mul",
        UDiv(_item) => "UDiv",
        SDiv(_item) => "SDiv",
        FAdd(_item) => "FAdd",
        FSub(_item) => "FSub",
        FMul(_item) => "FMul",
        FDiv(_item) => "FDiv",
        FRem(_item) => "FRem",
        Shl(_item) => "Shl",
        LShr(_item) => "LShr",
        AShr(_item) => "AShr",
        And(_item) => "And",
        Or(_item) => "Or",
        Xor(_item) => "Xor",
        Alloca(_item) => "Alloca",
        Load(_item) => "Load",
        Store(_item) => "Store",
        GetElementPtr(_item) => "GetElementPtr",
        Trunc(_item) => "Trunc",
        ZExt(_item) => "ZExt",
        SExt(_item) => "SExt",
        FPToUI(_item) => "FPToUI",
        FPToSI(_item) => "FPToSI",
        UIToFP(_item) => "UIToFP",
        SIToFP(_item) => "SIToFP",
        FPTrunc(_item) => "FPTrunc",
        FPExt(_item) => "FPExt",
        PtrToInt(_item) => "PtrToInt",
        IntToPtr(_item) => "IntToPtr",
        BitCast(_item) => "BitCast",
        ICmp(_item) => "ICmp",
        FCmp(_item) => "FCmp",
        Phi(_item) => "Phi",
        Call(_item) => "Call",
        Select(_item) => "Select",
        ExtractElement(_item) => "ExtractElement",
        InsertElement(_item) => "InsertElement",
        ShuffleVector(_item) => "ShuffleVector",
        ExtractValue(_item) => "ExtractValue",
        InsertValue(_item) => "InsertValue",
        LandingPad(_item) => "LandingPad",
        CatchPad(_item) => "CatchPad",
        CleanupPad(_item) => "CleanupPad",
        URem(_item) => "URem",
        SRem(_item) => "SRem",
        FNeg(_item) => "FNeg",
        Fence(_item) => "Fence",
        CmpXchg(_item) => "CmpXchg",
        AtomicRMW(_item) => "AtomicRMW",
        AddrSpaceCast(_item) => "AddrSpaceCast",
        Freeze(_item) => "Freeze",
        VAArg(_item) => "VAArg",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_seq() {
        let ir = r#"
        define i32 @main() {
        entry:
            %0 = add i32 1, 2
            %1 = sub i32 %0, 3
            ret i32 %1
        }
        "#;
        let module = llvm_ir::Module::from_ir_str(ir).unwrap();
        let res = extract_seq(&module).unwrap();
        let expected = vec![
            Element::Str("Add".to_string()),
            Element::Str("Sub".to_string()),
            Element::Str("Ret".to_string()),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_set() {
        let module = llvm_ir::Module::from_ir_path(PathBuf::from("../testdata/hello.ll")).unwrap();
        let res = extract_set(&module).unwrap();
        let expected = vec![
            Element::Str("Call".to_string()),
            Element::Str("Ret".to_string()),
        ];
        assert_eq!(res, expected);
    }
}
