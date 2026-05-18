use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_object_lit(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::ObjectLit(props, _ty) => {
                // Pre-evaluate all property keys and values on an empty stack
                let mut temp_slots = Vec::new();
                for prop in props {
                    match prop {
                        ObjectProp::KeyValue(_, val) => {
                            temp_slots.push((None, self.emit_expr_to_temp(val)));
                        }
                        ObjectProp::Computed(key_expr, val) => {
                            let key_slot = self.emit_expr_to_temp(key_expr);
                            let val_slot = self.emit_expr_to_temp(val);
                            temp_slots.push((Some(key_slot), val_slot));
                        }
                        ObjectProp::Method(_, _, _, _) => {
                            temp_slots.push((None, 0));
                        }
                        ObjectProp::Spread(inner) => {
                            temp_slots.push((None, self.emit_expr_to_temp(inner)));
                        }
                        ObjectProp::Getter(_, expr) => {
                            temp_slots.push((None, self.emit_expr_to_temp(expr)));
                        }
                        ObjectProp::Setter(_, expr) => {
                            temp_slots.push((None, self.emit_expr_to_temp(expr)));
                        }
                    }
                }

                // Use TsObject (extends LinkedHashMap) for prototype support
                let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                let ts_object_init = self.emitter.cp.add_method_ref(ts_object_class, "<init>".to_string(), "()V".to_string()).unwrap();
                let ts_set_property = self.emitter.cp.add_method_ref(ts_object_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
                self.code.push(Instruction::New(ts_object_class));
                self.code.push(Instruction::Dup);
                self.code.push(Instruction::Invokespecial(ts_object_init));

                for (prop, &(key_slot, val_slot)) in props.iter().zip(temp_slots.iter()) {
                    match prop {
                        ObjectProp::KeyValue(key, _) => {
                            self.code.push(Instruction::Dup);
                            let key_idx = self.emitter.cp.add_string(key).unwrap();
                            self.code.push(Instruction::Ldc_w(key_idx));
                            self.code.push(Instruction::Aload(val_slot));
                            self.code.push(Instruction::Invokevirtual(ts_set_property));
                        }
                        ObjectProp::Computed(_, _) => {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Aload(key_slot.unwrap()));
                            // Convert key to String
                            let string_class = self.emitter.cp.add_class("java/lang/String").unwrap();
                            let value_of = self.emitter.cp.add_method_ref(string_class, "valueOf".to_string(), "(Ljava/lang/Object;)Ljava/lang/String;".to_string()).unwrap();
                            self.code.push(Instruction::Invokestatic(value_of));
                            self.code.push(Instruction::Aload(val_slot));
                            self.code.push(Instruction::Invokevirtual(ts_set_property));
                        }
                        ObjectProp::Method(name, _args, _ret, _body) => {
                            self.code.push(Instruction::Dup);
                            let key_idx = self.emitter.cp.add_string(name).unwrap();
                            self.code.push(Instruction::Ldc_w(key_idx));
                            self.code.push(Instruction::Aconst_null);
                            self.code.push(Instruction::Invokevirtual(ts_set_property));
                        }
                        ObjectProp::Spread(_) => {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Aload(val_slot));
                            let linked_hash_map_class = self.emitter.cp.add_class("java/util/LinkedHashMap").unwrap();
                            let put_all = self.emitter.cp.add_method_ref(
                                linked_hash_map_class,
                                "putAll".to_string(),
                                "(Ljava/util/Map;)V".to_string()
                            ).unwrap();
                            self.code.push(Instruction::Invokevirtual(put_all));
                        }
                        ObjectProp::Getter(name, _) => {
                            self.code.push(Instruction::Dup);
                            let key_idx = self.emitter.cp.add_string(name).unwrap();
                            self.code.push(Instruction::Ldc_w(key_idx));
                            self.code.push(Instruction::Aload(val_slot));
                            let define_getter = self.emitter.cp.add_method_ref(ts_object_class, "defineGetter".to_string(), "(Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(define_getter));
                        }
                        ObjectProp::Setter(name, _) => {
                            self.code.push(Instruction::Dup);
                            let key_idx = self.emitter.cp.add_string(name).unwrap();
                            self.code.push(Instruction::Ldc_w(key_idx));
                            self.code.push(Instruction::Aload(val_slot));
                            let define_setter = self.emitter.cp.add_method_ref(ts_object_class, "defineSetter".to_string(), "(Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(define_setter));
                        }
                    }
                }

                // Free temp slots
                for &(key_slot, val_slot) in temp_slots.iter().rev() {
                    if val_slot > 0 {
                        self.emitter.local_slot -= 1;
                    }
                    if key_slot.is_some() {
                        self.emitter.local_slot -= 1;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
