use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_yield(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Yield(arg_opt, delegate) => {
                let queue_var = HirExpr::Var(
                    "__queue".to_string(),
                    Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()),
                );

                if !delegate {
                    // 1. Load queue
                    self.emit_queue_load(&queue_var);

                    // 2. Evaluate and box yield argument
                    if let Some(arg) = arg_opt {
                        self.emit_expr(arg);
                        let ty = arg.get_type();
                        self.box_if_needed(&ty);
                    } else {
                        self.code.push(Instruction::Aconst_null);
                    }

                    // 3. Invoke LinkedBlockingQueue.put(Ljava/lang/Object;)V
                    let queue_class = self.emitter.cp.add_class("java/util/concurrent/LinkedBlockingQueue").unwrap();
                    let put_m = self.emitter.cp.add_method_ref(
                        queue_class,
                        "put".to_string(),
                        "(Ljava/lang/Object;)V".to_string(),
                    ).unwrap();
                    self.code.push(Instruction::Invokevirtual(put_m));

                    // 4. Yield expression evaluates to undefined/null
                    self.code.push(Instruction::Aconst_null);
                } else {
                    // yield* delegate: loop over iterable and put each to queue
                    if let Some(delegate_expr) = arg_opt {
                        self.emit_expr(delegate_expr);
                        let iter_class = self.emitter.cp.add_class("java/lang/Iterable").unwrap();
                        let iterator_m = self.emitter.cp.add_interface_method_ref(
                            iter_class,
                            "iterator".to_string(),
                            "()Ljava/util/Iterator;".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokeinterface(iterator_m, 1));

                        let iter_slot = self.emitter.local_slot;
                        self.emitter.local_slot += 1;
                        self.code.push(Instruction::Astore(iter_slot));

                        let start_pc = self.code.len();
                        self.code.push(Instruction::Aload(iter_slot));

                        let iterator_class = self.emitter.cp.add_class("java/util/Iterator").unwrap();
                        let has_next_m = self.emitter.cp.add_interface_method_ref(
                            iterator_class,
                            "hasNext".to_string(),
                            "()Z".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokeinterface(has_next_m, 1));

                        let ifeq_idx = self.code.len();
                        self.code.push(Instruction::Ifeq(0));

                        // loop body: put(iter.next())
                        self.emit_queue_load(&queue_var);

                        self.code.push(Instruction::Aload(iter_slot));
                        let next_m = self.emitter.cp.add_interface_method_ref(
                            iterator_class,
                            "next".to_string(),
                            "()Ljava/lang/Object;".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokeinterface(next_m, 1));

                        let queue_class = self.emitter.cp.add_class("java/util/concurrent/LinkedBlockingQueue").unwrap();
                        let put_m = self.emitter.cp.add_method_ref(
                            queue_class,
                            "put".to_string(),
                            "(Ljava/lang/Object;)V".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokevirtual(put_m));

                        self.code.push(Instruction::Goto(start_pc as u16));

                        let end_pc = self.code.len();
                        self.code[ifeq_idx] = Instruction::Ifeq(end_pc as u16);

                        self.emitter.local_slot -= 1; // free iter_slot
                    }
                    self.code.push(Instruction::Aconst_null);
                }
            }
            _ => unreachable!(),
        }
    }

    fn emit_queue_load(&mut self, queue_var: &HirExpr) {
        if let Some(ref class_path) = self.emitter.class_path {
            if class_path.contains("/Lambda$") {
                self.code.push(Instruction::Aload_0);
                let class_idx = self.emitter.cp.add_class(class_path).unwrap();
                let field_idx = self.emitter.cp.add_field_ref(
                    class_idx,
                    "__queue".to_string(),
                    "Ljava/lang/Object;".to_string(),
                ).unwrap();
                self.code.push(Instruction::Getfield(field_idx));
                let queue_class = self.emitter.cp.add_class("java/util/concurrent/LinkedBlockingQueue").unwrap();
                self.code.push(Instruction::Checkcast(queue_class));
            } else {
                self.emit_expr(queue_var);
            }
        } else {
            self.emit_expr(queue_var);
        }
    }
}
