use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_expr_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Expr(expr) => {
                            self.emit_expr(expr);
                            let resolved_ty = self.resolve_type(expr);
                            match resolved_ty {
                                Type::Double => self.code.push(Instruction::Pop2),
                                Type::Void => {},
                                _ => self.code.push(Instruction::Pop),
                            }
                        }
            _ => unreachable!(),
        }
    }
}
