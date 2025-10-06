use std::collections::{HashMap, HashSet};

use crate::birthmarks::Element;
use crate::extractors::Extractor;
use crate::Result;

pub(super) struct SeqExtractor {
    opcodes: Vec<Element>,
}

impl SeqExtractor {
    pub fn new() -> Self {
        Self { opcodes: vec![] }
    }
}

impl Extractor for SeqExtractor {
    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        crate::birthmarks::BirthmarkType::OpSeq
    }

    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }
    
    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }
    
    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        let opcode = Element::Str(instruction_to_str(instr));
        self.opcodes.push(opcode.clone());
        Ok(Some(opcode))
    }
    
    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        let opcode = Element::Str(terminator_to_str(term));
        self.opcodes.push(opcode.clone());
        Ok(self.opcodes.clone())
    }
    
    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone())
    }
    
    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone())
    }
    
    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }
    
    fn clear(&mut self) {
        self.opcodes.clear();
    }
}

pub(super) struct SetExtractor {
    opcodes: HashSet<Element>,
}

impl SetExtractor {
    pub fn new() -> Self {
        Self { opcodes: HashSet::new() }
    }
}

impl Extractor for SetExtractor {
    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        crate::birthmarks::BirthmarkType::OpSet
    }
    
    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }
    
    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }
    
    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        let opcode = Element::Str(instruction_to_str(instr));
        self.opcodes.insert(opcode.clone());
        Ok(Some(opcode))
    }
    
    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        let opcode = Element::Str(terminator_to_str(term));
        self.opcodes.insert(opcode.clone());
        Ok(self.opcodes.clone().into_iter().collect())
    }
    
    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone().into_iter().collect())
    }
    
    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone().into_iter().collect())
    }
    
    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }
    
    fn clear(&mut self) {
        self.opcodes.clear();
    }
}

pub(super) struct FreqExtractor {
    opcodes: HashMap<String, usize>,
}

impl FreqExtractor {
    pub fn new() -> Self {
        Self { opcodes: HashMap::new() }
    }
}

impl Extractor for FreqExtractor {
    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        crate::birthmarks::BirthmarkType::OpFreq
    }

    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }

    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        let opcode = instruction_to_str(instr);
        *self.opcodes.entry(opcode.clone()).or_insert(0) += 1;
        let v = *self.opcodes.get(&opcode).unwrap_or(&0);
        Ok(Some(Element::Freq(v, opcode)))
    }

    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        let opcode = terminator_to_str(term);
        *self.opcodes.entry(opcode.clone()).or_insert(0) += 1;
        Ok(self.opcodes.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.opcodes.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }

    fn clear(&mut self) {
        self.opcodes.clear();
    }
}

pub(super) struct KGramExtractor {
    n: usize,
    current: KGram,
    kgrams: Vec<KGram>,
}

impl KGramExtractor {
    pub fn new(n: usize) -> Self {
        Self { n, current: KGram::new(n), kgrams: vec![] }
    }
}

impl Extractor for KGramExtractor {
    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        use crate::birthmarks::BirthmarkType::*;
        match self.n {
            1 => UniGram,
            2 => BiGram,
            3 => TriGram,
            4 => TetraGram,
            5 => PentaGram,
            6 => HexaGram,
            7 => HeptaGram,
            8 => OctaGram,
            _ => panic!("Unsupported k-gram size: {}", self.n),
        }
    }

    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }

    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        self.current.push(instruction_to_str(instr));
        if self.current.is_valid() {
            self.kgrams.push(self.current.clone());
            Ok(Some(self.current.to_elem()))
        } else {
            Ok(None)
        }
    }

    fn visit_bb_end(&mut self, term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        self.current.push(terminator_to_str(term));
        if self.current.is_valid() {
            self.kgrams.push(self.current.clone());
        }
        Ok(self.kgrams.clone().into_iter().map(|k| k.to_elem()).collect())
    }

    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.kgrams.clone().into_iter().map(|k| k.to_elem()).collect())
    }

    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.kgrams.clone().into_iter().map(|k| k.to_elem()).collect())
    }

    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }

    fn clear(&mut self) {
        self.current = KGram::new(self.n);
        self.kgrams.clear();
    }
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
    use crate::{birthmarks::BirthmarkType, extractors::{extract, Mode}};

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
        let birthmarks = extract(&module, PathBuf::from("<memory>"), &BirthmarkType::OpSeq, &Mode::File).unwrap();
        assert_eq!(birthmarks.len(), 1);
        let res = &birthmarks[0].elements;
        let expected = vec![
            Element::Str("Add".to_string()),
            Element::Str("Sub".to_string()),
            Element::Str("Ret".to_string()),
        ];
        assert_eq!(res, &expected);
    }

    #[test]
    fn test_set() {
        let path = PathBuf::from("../testdata/hello.ll");
        let module = llvm_ir::Module::from_ir_path(&path).unwrap();
        let birthmarks = extract(&module, &path, &BirthmarkType::OpSet, &Mode::File).unwrap();
        assert_eq!(birthmarks.len(), 1);
        let mut res = birthmarks[0].elements.clone();
        res.sort();
        let expected = vec![
            Element::Str("Call".to_string()),
            Element::Str("Ret".to_string()),
        ];
        assert_eq!(res, expected);
    }
}
