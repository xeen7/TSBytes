use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use super::super::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(super) fn emit_dynamic_import(&mut self, expr: &HirExpr) {
        if let HirExpr::DynamicImport(path_expr) = expr {
            // Emit the dynamic import module path string
            self.emit_expr(path_expr);
            
            // Class.forName(String)
            let class_idx = self.emitter.cp.add_class("java/lang/Class").unwrap();
            let for_name_idx = self.emitter.cp.add_method_ref(
                class_idx, "forName", "(Ljava/lang/String;)Ljava/lang/Class;"
            ).unwrap();
            self.code.push(Instruction::Invokestatic(for_name_idx));
            
            // Wrap in CompletableFuture.completedFuture(Object)
            let future_class_idx = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
            let completed_idx = self.emitter.cp.add_method_ref(
                future_class_idx, "completedFuture", "(Ljava/lang/Object;)Ljava/util/concurrent/CompletableFuture;"
            ).unwrap();
            self.code.push(Instruction::Invokestatic(completed_idx));
        }
    }
}
