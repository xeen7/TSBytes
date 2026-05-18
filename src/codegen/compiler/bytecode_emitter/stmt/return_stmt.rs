use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_return_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Return(expr_opt) => {
                            if let Some(expr) = expr_opt {
                                self.emit_expr(expr);
                                let resolved_ty = self.resolve_type(expr);
                                self.box_if_needed(&resolved_ty);
                            } else {
                                self.code.push(Instruction::Aconst_null);
                            }
                            
                            if self.is_async {
                                let cf_class = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
                                let completed = self.emitter.cp.add_method_ref(
                                    cf_class, 
                                    "completedFuture".to_string(), 
                                    "(Ljava/lang/Object;)Ljava/util/concurrent/CompletableFuture;".to_string()
                                ).unwrap();
                                self.code.push(Instruction::Invokestatic(completed));
                            }
                            
                            self.code.push(Instruction::Areturn);
                        }
            _ => unreachable!(),
        }
    }
}
