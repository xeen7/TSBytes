use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_arrow(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Arrow(_, _, _, _) => {
                            // Arrow functions are lowered by closure_pass before reaching here
                            self.code.push(Instruction::Aconst_null); // stub
                        }
            _ => unreachable!(),
        }
    }
}
