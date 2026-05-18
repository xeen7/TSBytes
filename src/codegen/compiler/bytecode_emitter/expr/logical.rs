use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_logical(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::LogicalAnd(left, right, _ty) => {
                self.emit_expr(left);
                let left_ty = self.resolve_type(left);
                self.box_if_needed(&left_ty);
                
                self.code.push(Instruction::Dup);
                
                let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                let is_truthy_method = self.emitter.cp.add_method_ref(
                    ts_object_class,
                    "isTruthy",
                    "(Ljava/lang/Object;)Z",
                ).unwrap();
                self.code.push(Instruction::Invokestatic(is_truthy_method));
                
                let ifeq_idx = self.code.len();
                self.code.push(Instruction::Ifeq(0)); // placeholder
                
                self.code.push(Instruction::Pop);
                self.emit_expr(right);
                let right_ty = self.resolve_type(right);
                self.box_if_needed(&right_ty);
                
                self.code[ifeq_idx] = Instruction::Ifeq(self.code.len() as u16);
            }
            HirExpr::LogicalOr(left, right, _ty) => {
                self.emit_expr(left);
                let left_ty = self.resolve_type(left);
                self.box_if_needed(&left_ty);
                
                self.code.push(Instruction::Dup);
                
                let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                let is_truthy_method = self.emitter.cp.add_method_ref(
                    ts_object_class,
                    "isTruthy",
                    "(Ljava/lang/Object;)Z",
                ).unwrap();
                self.code.push(Instruction::Invokestatic(is_truthy_method));
                
                let ifne_idx = self.code.len();
                self.code.push(Instruction::Ifne(0)); // placeholder
                
                self.code.push(Instruction::Pop);
                self.emit_expr(right);
                let right_ty = self.resolve_type(right);
                self.box_if_needed(&right_ty);
                
                self.code[ifne_idx] = Instruction::Ifne(self.code.len() as u16);
            }
            _ => unreachable!(),
        }
    }
}
