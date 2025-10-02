use llvm_ir::{Constant, Name};
use either::Either;

use crate::birthmarks::Element;
use crate::Result;

pub(super) fn extract_names(module: &llvm_ir::Module) -> Result<Vec<Element>> {
    let mut names = Vec::new();
    for func in &module.functions {
        for bb in &func.basic_blocks {
            for inst in &bb.instrs {
                if let llvm_ir::Instruction::Call(call) = inst {
                    if let Some(fname) = extract_called_name(&call) {
                        names.push(Element::Str(fname));
                    }
                }
            }
        }
        names.push(Element::Str(func.name.clone()));
    }
    Ok(names)
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
