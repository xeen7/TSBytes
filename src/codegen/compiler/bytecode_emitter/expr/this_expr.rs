use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_this_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::This(_ty) => {
                            self.code.push(Instruction::Aload_0);
                        }
            _ => unreachable!(),
        }
    }
}
