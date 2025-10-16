use crate::birthmarks::Element;
use crate::Result;

#[allow(dead_code)]
pub(super) fn extract(module: &llvm_ir::Module) -> Result<Vec<Element>> {
    let mut operands = Vec::new();
    for func in &module.functions {
        for bb in &func.basic_blocks {
            for instr in &bb.instrs {
                let operand = format!("{:?}", instr);
                let operand = operand.split_whitespace().next().unwrap_or("");
                if !operand.is_empty() {
                    operands.push(Element::Str(operand.to_string()));
                }
            }
        }
    }
    Ok(operands)
}