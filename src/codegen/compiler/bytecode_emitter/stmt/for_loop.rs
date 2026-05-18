use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_for_loop(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::ForOf(var, ty, iterable, body) => {
                            // iterable.iterator() -> while(iter.hasNext()) { var = iter.next(); body }
                            self.emit_expr(iterable);
                            let iter_class = self.emitter.cp.add_class("java/lang/Iterable").unwrap();
                            let iterator_m = self.emitter.cp.add_interface_method_ref(iter_class, "iterator".to_string(), "()Ljava/util/Iterator;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(iterator_m, 1));
                            let iter_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.code.push(Instruction::Astore(iter_slot));
                            let start_pc = self.code.len();
                            self.code.push(Instruction::Aload(iter_slot));
                            let has_next_class = self.emitter.cp.add_class("java/util/Iterator").unwrap();
                            let has_next = self.emitter.cp.add_interface_method_ref(has_next_class, "hasNext".to_string(), "()Z".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(has_next, 1));
                            let ifeq_idx = self.code.len();
                            self.code.push(Instruction::Ifeq(0));
                            
                            let label = self.pending_label.take();
                            self.loop_stack.push((label, start_pc, Vec::new(), Vec::new()));
                            self.code.push(Instruction::Aload(iter_slot));
                            let next_m = self.emitter.cp.add_interface_method_ref(has_next_class, "next".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(next_m, 1));
                            let var_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.emitter.id_registry.insert(var.clone(), (var_slot, ty.clone()));
                            self.code.push(Instruction::Astore(var_slot));
                            for s in body { self.emit_stmt(s); }
                            
                            let continue_pc = start_pc;
                            self.code.push(Instruction::Goto(continue_pc as u16));
                            
                            let end_pc = self.code.len();
                            self.code[ifeq_idx] = Instruction::Ifeq(end_pc as u16);
                            
                            let (_, _, break_indices, continue_indices) = self.loop_stack.pop().unwrap();
                            for idx in break_indices { self.code[idx] = Instruction::Goto(end_pc as u16); }
                            for idx in continue_indices { self.code[idx] = Instruction::Goto(continue_pc as u16); }
                        }
            HirStmt::ForAwaitOf(var, ty, iterable, body) => {
                            self.emit_expr(iterable);
                            let iter_class = self.emitter.cp.add_class("java/lang/Iterable").unwrap();
                            let iterator_m = self.emitter.cp.add_interface_method_ref(iter_class, "iterator".to_string(), "()Ljava/util/Iterator;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(iterator_m, 1));
                            let iter_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.code.push(Instruction::Astore(iter_slot));
                            let start_pc = self.code.len();
                            self.code.push(Instruction::Aload(iter_slot));
                            let has_next_class = self.emitter.cp.add_class("java/util/Iterator").unwrap();
                            let has_next = self.emitter.cp.add_interface_method_ref(has_next_class, "hasNext".to_string(), "()Z".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(has_next, 1));
                            let ifeq_idx = self.code.len();
                            self.code.push(Instruction::Ifeq(0));
                            
                            let label = self.pending_label.take();
                            self.loop_stack.push((label, start_pc, Vec::new(), Vec::new()));
                            self.code.push(Instruction::Aload(iter_slot));
                            let next_m = self.emitter.cp.add_interface_method_ref(has_next_class, "next".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(next_m, 1));
                            
                            // Instanceof CompletableFuture
                            let cf_class = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Instanceof(cf_class));
                            let if_not_cf_idx = self.code.len();
                            self.code.push(Instruction::Ifeq(0));
                            
                            self.code.push(Instruction::Checkcast(cf_class));
                            let join_m = self.emitter.cp.add_method_ref(cf_class, "join".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(join_m));
                            
                            let goto_end_idx = self.code.len();
                            self.code.push(Instruction::Goto(0));
                            
                            let else_pc = self.code.len();
                            self.code[if_not_cf_idx] = Instruction::Ifeq(else_pc as u16);
                            
                            let merge_pc = self.code.len();
                            self.code[goto_end_idx] = Instruction::Goto(merge_pc as u16);
                            
                            let var_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.emitter.id_registry.insert(var.clone(), (var_slot, ty.clone()));
                            self.code.push(Instruction::Astore(var_slot));
                            for s in body { self.emit_stmt(s); }
                            
                            let continue_pc = start_pc;
                            self.code.push(Instruction::Goto(continue_pc as u16));
                            
                            let end_pc = self.code.len();
                            self.code[ifeq_idx] = Instruction::Ifeq(end_pc as u16);
                            
                            let (_, _, break_indices, continue_indices) = self.loop_stack.pop().unwrap();
                            for idx in break_indices { self.code[idx] = Instruction::Goto(end_pc as u16); }
                            for idx in continue_indices { self.code[idx] = Instruction::Goto(continue_pc as u16); }
                        }
            HirStmt::ForIn(var, obj_expr, body) => {
                            // for (const key in obj) → obj.allKeys().iterator() loop
                            self.emit_expr(obj_expr);
                            let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                            let all_keys = self.emitter.cp.add_method_ref(ts_object_class, "allKeys".to_string(), "()Ljava/util/Set;".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(all_keys));
                            let set_class = self.emitter.cp.add_class("java/util/Set").unwrap();
                            let set_iterator = self.emitter.cp.add_interface_method_ref(set_class, "iterator".to_string(), "()Ljava/util/Iterator;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(set_iterator, 1));
                            let iter_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.code.push(Instruction::Astore(iter_slot));
                            let start_pc = self.code.len();
                            self.code.push(Instruction::Aload(iter_slot));
                            let iter_class = self.emitter.cp.add_class("java/util/Iterator").unwrap();
                            let has_next = self.emitter.cp.add_interface_method_ref(iter_class, "hasNext".to_string(), "()Z".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(has_next, 1));
                            let ifeq_idx = self.code.len();
                            self.code.push(Instruction::Ifeq(0));
                            
                            let label = self.pending_label.take();
                            self.loop_stack.push((label, start_pc, Vec::new(), Vec::new()));
                            self.code.push(Instruction::Aload(iter_slot));
                            let next_m = self.emitter.cp.add_interface_method_ref(iter_class, "next".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
                            self.code.push(Instruction::Invokeinterface(next_m, 1));
                            let var_slot = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.emitter.id_registry.insert(var.clone(), (var_slot, Type::StringTy));
                            self.code.push(Instruction::Astore(var_slot));
                            for s in body { self.emit_stmt(s); }
                            
                            let continue_pc = start_pc;
                            self.code.push(Instruction::Goto(continue_pc as u16));
                            
                            let end_pc = self.code.len();
                            self.code[ifeq_idx] = Instruction::Ifeq(end_pc as u16);
                            
                            let (_, _, break_indices, continue_indices) = self.loop_stack.pop().unwrap();
                            for idx in break_indices { self.code[idx] = Instruction::Goto(end_pc as u16); }
                            for idx in continue_indices { self.code[idx] = Instruction::Goto(continue_pc as u16); }
                        }
            _ => unreachable!(),
        }
    }
}
