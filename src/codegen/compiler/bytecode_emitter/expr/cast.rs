use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_cast(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Cast(expr, ty) => {
                            self.emit_expr(expr);
                            if let Type::Class(c) = ty {
                                let class_idx = self.emitter.cp.add_class(c).unwrap();
                                self.code.push(Instruction::Checkcast(class_idx));
                            } else {
                                self.unbox_if_needed(ty);
                            }
                        }
            _ => unreachable!(),
        }
    }
}
