use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_ternary(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Ternary(cond, then_expr, else_expr, ty) => {
                             self.emit_cond_truthiness(cond);
                             let ifeq_idx = self.code.len();
                             self.code.push(Instruction::Ifeq(0));
                             
                             self.emit_expr(then_expr);
                             let then_ty = self.resolve_type(then_expr);
                             self.coerce_to_expected_type(&then_ty, ty);
                             
                             let goto_idx = self.code.len();
                             self.code.push(Instruction::Goto(0));
                             self.code[ifeq_idx] = Instruction::Ifeq(self.code.len() as u16);
                             
                             self.emit_expr(else_expr);
                             let else_ty = self.resolve_type(else_expr);
                             self.coerce_to_expected_type(&else_ty, ty);
                             
                             self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                         }
            _ => unreachable!(),
        }
    }
}
