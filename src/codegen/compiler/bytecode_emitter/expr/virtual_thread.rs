use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_virtual_thread_spawn(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::VirtualThreadSpawn(closure) => {
                // 1. Emit the closure instantiation (Supplier)
                self.emit_expr(closure);

                // 2. Call Executors.newVirtualThreadPerTaskExecutor()
                let executors_class = self.emitter.cp.add_class("java/util/concurrent/Executors").unwrap();
                let new_vt = self.emitter.cp.add_method_ref(
                    executors_class,
                    "newVirtualThreadPerTaskExecutor".to_string(),
                    "()Ljava/util/concurrent/ExecutorService;".to_string()
                ).unwrap();
                self.code.push(Instruction::Invokestatic(new_vt));

                // 3. Call CompletableFuture.supplyAsync(Supplier, Executor)
                let cf_class = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
                let supply_async = self.emitter.cp.add_method_ref(
                    cf_class,
                    "supplyAsync".to_string(),
                    "(Ljava/util/function/Supplier;Ljava/util/concurrent/Executor;)Ljava/util/concurrent/CompletableFuture;".to_string()
                ).unwrap();
                self.code.push(Instruction::Invokestatic(supply_async));
            }
            _ => unreachable!(),
        }
    }
}
