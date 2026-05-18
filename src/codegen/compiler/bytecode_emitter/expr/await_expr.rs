use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_await_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Await(inner, _ty) => {
                            self.emit_expr(inner);
                            // cast to CompletableFuture
                            let cf_class = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
                            self.code.push(Instruction::Checkcast(cf_class));
                            // call CompletableFuture.join()
                            let join_m = self.emitter.cp.add_method_ref(cf_class, "join".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(join_m));
                        }
            _ => unreachable!(),
        }
    }
}
