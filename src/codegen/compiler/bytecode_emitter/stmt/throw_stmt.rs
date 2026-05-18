use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_throw_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Throw(expr) => {
                self.emit_expr(expr);
                let expr_ty = self.resolve_type(expr);
                self.box_if_needed(&expr_ty);

                let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                let wrap_throw = self.emitter.cp.add_method_ref(runtime_class, "wrapThrow".to_string(), "(Ljava/lang/Object;)Ljava/lang/RuntimeException;".to_string()).unwrap();
                self.code.push(Instruction::Invokestatic(wrap_throw));
                self.code.push(Instruction::Athrow);
            }
            _ => unreachable!(),
        }
    }
}
