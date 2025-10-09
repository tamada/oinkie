use std::collections::HashMap;

use llvm_ir::{Constant, Name};
use either::Either;

use crate::birthmarks::Element;
use crate::extractors::Extractor;
use crate::Result;

pub(super) struct SeqNames {
    names: Vec<Element>,
}

impl SeqNames {
    pub fn new() -> Self {
        Self { names: vec![] }
    }
}

impl Extractor for SeqNames {
    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        crate::birthmarks::BirthmarkType::Sfc
    }

    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }

    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        if let llvm_ir::Instruction::Call(call) = instr {
            if let Some(fname) = extract_called_name(&call) {
                let r = Element::Str(fname.clone());
                self.names.push(r.clone());
                Ok(Some(r))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn visit_bb_end(&mut self, _term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        Ok(self.names.clone())
    }

    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.names.clone())
    }

    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.names.clone())
    }

    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }

    fn clear(&mut self) {
        self.names.clear();
    }
}

pub(super) struct FreqNames {
    freq: HashMap<String, usize>,
}

impl FreqNames {
    pub fn new() -> Self {
        Self { freq: HashMap::new() }
    }
}

impl Extractor for FreqNames {
    fn btype(&self) -> crate::birthmarks::BirthmarkType {
        crate::birthmarks::BirthmarkType::Ffc
    }

    fn visit(&mut self, _module: &llvm_ir::Module, _path: &std::path::PathBuf) {
    }

    fn visit_func(&mut self, _func: &llvm_ir::Function) {
    }

    fn visit_bb(&mut self, _bb: &llvm_ir::basicblock::BasicBlock) {
    }

    fn visit_inst(&mut self, instr: &llvm_ir::Instruction) -> Result<Option<Element>> {
        if let llvm_ir::Instruction::Call(call) = instr {
            if let Some(fname) = extract_called_name(&call) {
                *self.freq.entry(fname.clone()).or_insert(0) += 1;
                Ok(Some(Element::Freq(*self.freq.get(&fname).unwrap(), fname.clone())))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn visit_bb_end(&mut self, _term: &llvm_ir::terminator::Terminator) -> Result<Vec<Element>> {
        Ok(self.freq.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn visit_func_end(&mut self, _func: &llvm_ir::Function) -> Result<Vec<Element>> {
        Ok(self.freq.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn visit_end(&mut self, _module: &llvm_ir::Module) -> Result<Vec<Element>> {
        Ok(self.freq.clone().into_iter().map(|(e, i)| Element::Freq(i, e)).collect())
    }

    fn finish(&self) -> Result<Vec<crate::birthmarks::Birthmark>> {
        Ok(vec![])
    }

    fn clear(&mut self) {
        self.freq.clear();
    }
}

fn extract_called_name(call: &llvm_ir::instruction::Call) -> Option<String> {
    match &call.function {
        Either::Left(assembly) => {
            match &assembly.ty.as_ref() {
                _ => None,
                // llvm_ir::Type::VoidType => todo!(),
                // llvm_ir::Type::IntegerType { bits } => todo!(),
                // llvm_ir::Type::PointerType { addr_space } => todo!(),
                // llvm_ir::Type::FPType(fptype) => todo!(),
                // llvm_ir::Type::FuncType { result_type, param_types, is_var_arg } => todo!(),
                // llvm_ir::Type::VectorType { element_type, num_elements, scalable } => todo!(),
                // llvm_ir::Type::ArrayType { element_type, num_elements } => todo!(),
                // llvm_ir::Type::StructType { element_types, is_packed } => todo!(),
                // llvm_ir::Type::NamedStructType { name } => todo!(),
                // llvm_ir::Type::X86_MMXType => todo!(),
                // llvm_ir::Type::X86_AMXType => todo!(),
                // llvm_ir::Type::MetadataType => todo!(),
                // llvm_ir::Type::LabelType => todo!(),
                // llvm_ir::Type::TokenType => todo!(),
                // llvm_ir::Type::TargetExtType => todo!(),
            }
        },
        Either::Right(llvm_ir::Operand::ConstantOperand(c)) => match c.as_ref() {
            Constant::GlobalReference { name, ..} => match name {
                Name::Name(n) => Some(n.as_str().to_string()),
                Name::Number(_n) => None,
            },
            _ => None,
            // Constant::Int { bits, value } => None,
            // Constant::Float(f) => None,
            // Constant::Null(type_ref) => todo!(),
            // Constant::AggregateZero(type_ref) => todo!(),
            // Constant::Struct { name, values, is_packed } => todo!(),
            // Constant::Array { element_type, elements } => todo!(),
            // Constant::Vector(constant_refs) => todo!(),
            // Constant::Undef(type_ref) => todo!(),
            // Constant::Poison(type_ref) => todo!(),
            // Constant::BlockAddress => todo!(),
            // Constant::TokenNone => todo!(),
            // Constant::PtrAuth { ptr, key, disc, addr_disc } => todo!(),
            // Constant::Add(add) => todo!(),
            // Constant::Sub(sub) => todo!(),
            // Constant::Mul(mul) => todo!(),
            // Constant::Xor(xor) => todo!(),
            // Constant::ExtractElement(extract_element) => todo!(),
            // Constant::InsertElement(insert_element) => todo!(),
            // Constant::ShuffleVector(shuffle_vector) => todo!(),
            // Constant::GetElementPtr(get_element_ptr) => todo!(),
            // Constant::Trunc(trunc) => todo!(),
            // Constant::PtrToInt(ptr_to_int) => todo!(),
            // Constant::IntToPtr(int_to_ptr) => todo!(),
            // Constant::BitCast(bit_cast) => todo!(),
            // Constant::AddrSpaceCast(addr_space_cast) => todo!(),
        },
        Either::Right(llvm_ir::Operand::LocalOperand { name, .. }) => match name {
            Name::Name(n) => Some(n.as_str().to_string()),
            Name::Number(_n) => None,
        },
        _ => None,
    }
}
