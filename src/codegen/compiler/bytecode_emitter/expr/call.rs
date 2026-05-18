use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_call(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Call(name, args, ty) => {
                // 1. Pre-evaluate args on an empty stack
                let mut temp_slots = Vec::new();
                for arg in args {
                    temp_slots.push(self.emit_expr_to_temp(arg));
                }

                if self.emitter.id_registry.contains_key(name) {
                    let (slot, _) = self.emitter.id_registry.get(name).unwrap().clone();
                    self.code.push(Instruction::Aload(slot));
                    
                    // Pack args
                    self.code.push(Instruction::Bipush(args.len() as i8));
                    let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                    self.code.push(Instruction::Anewarray(obj_class));
                    for (i, &temp_slot) in temp_slots.iter().enumerate() {
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Bipush(i as i8));
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Aastore);
                    }
                    
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                    let invoke_lambda = self.emitter.cp.add_method_ref(
                        runtime_class, "invokeLambda".to_string(), "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                    ).unwrap();
                    self.code.push(Instruction::Invokestatic(invoke_lambda));
                    self.unbox_if_needed(ty);

                    // Free temp slots
                    for _ in &temp_slots {
                        self.emitter.local_slot -= 1;
                    }
                    return;
                }

                let has_spread = args.iter().any(|a| matches!(a, HirExpr::Spread(_)));
                if has_spread {
                    // Use ArrayList to pack args with spread
                    self.code.push(Instruction::New(self.emitter.array_list_class));
                    self.code.push(Instruction::Dup);
                    self.code.push(Instruction::Invokespecial(self.emitter.array_list_init));
                    for (arg, &temp_slot) in args.iter().zip(temp_slots.iter()) {
                        if let HirExpr::Spread(_) = arg {
                            let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                            let spread_arr = self.emitter.cp.add_method_ref(
                                runtime_class, "spreadArray".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)V".to_string()
                            ).unwrap();
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Invokestatic(spread_arr));
                        } else {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Invokevirtual(self.emitter.array_list_add));
                            self.code.push(Instruction::Pop);
                        }
                    }
                    // Convert ArrayList to Object[]
                    let array_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    let to_array = self.emitter.cp.add_method_ref(array_class, "toArray".to_string(), "()[Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(to_array));
                } else {
                    // Fixed-size Object[] array
                    self.code.push(Instruction::Bipush(args.len() as i8));
                    let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                    self.code.push(Instruction::Anewarray(obj_class));
                    for (i, &temp_slot) in temp_slots.iter().enumerate() {
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Bipush(i as i8));
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Aastore);
                    }
                }
                
                let cp_name = match name.as_str() {
                    "parseInt" | "parseFloat" | "isNaN" | "isFinite" | "encodeURIComponent" | "decodeURIComponent" | "String" | "Number" | "Boolean" | "fetch" | "startMockServer" | "stopMockServer" => {
                        "com/tsdroid/runtime/TsRuntime".to_string()
                    }
                    _ => {
                        if let Some(ref path) = self.emitter.class_path {
                            if path.contains('/') {
                                let package = path.rsplit_once('/').unwrap().0;
                                format!("{}/App", package)
                            } else {
                                "com/tsdroid/app/App".to_string()
                            }
                        } else {
                            "com/tsdroid/app/App".to_string()
                        }
                    }
                };
                let class_idx = self.emitter.cp.add_class(&cp_name).unwrap();
                let method_idx = self.emitter.cp.add_method_ref(class_idx, name.clone(), "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                self.code.push(Instruction::Invokestatic(method_idx));
                self.unbox_if_needed(ty);

                // Free temp slots
                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn emit_dynamic_call(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::DynamicCall(callee, args, ty) => {
                self.emit_expr(callee);
                let callee_ty = callee.get_type();
                self.box_if_needed(&callee_ty);
                
                let callee_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                self.code.push(Instruction::Astore(callee_slot));
                
                let mut temp_slots = Vec::new();
                for arg in args {
                    temp_slots.push(self.emit_expr_to_temp(arg));
                }
                
                self.code.push(Instruction::Aload(callee_slot));
                
                self.code.push(Instruction::Bipush(args.len() as i8));
                let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                self.code.push(Instruction::Anewarray(obj_class));
                for (i, &temp_slot) in temp_slots.iter().enumerate() {
                    self.code.push(Instruction::Dup);
                    self.code.push(Instruction::Bipush(i as i8));
                    self.code.push(Instruction::Aload(temp_slot));
                    self.code.push(Instruction::Aastore);
                }
                
                let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                let invoke_lambda = self.emitter.cp.add_method_ref(
                    runtime_class, "invokeLambda".to_string(), "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                ).unwrap();
                self.code.push(Instruction::Invokestatic(invoke_lambda));
                
                self.unbox_if_needed(ty);
                
                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
                
                self.emitter.local_slot -= 1;
            }
            _ => unreachable!(),
        }
    }
}
