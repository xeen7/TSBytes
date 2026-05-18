use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_optional_chain(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::OptionalChain(obj, field, ty) => {
                // 1. Emit object
                self.emit_expr(obj);
                
                // 2. Allocate temporary slot to store the object
                let temp_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                
                // 3. Store object in the temporary slot
                self.code.push(Instruction::Astore(temp_slot));
                
                // 4. Load it for the null check
                self.code.push(Instruction::Aload(temp_slot));
                
                // 5. Jump to null handler if the loaded object is null
                let ifnull_idx = self.code.len();
                self.code.push(Instruction::Ifnull(0)); // placeholder
                
                // 6. Object is NOT null: load it again and perform property lookup
                self.code.push(Instruction::Aload(temp_slot));
                
                let obj_ty = self.resolve_type(obj);
                if matches!(obj_ty, Type::Object(_)) || obj_ty == Type::Any || matches!(obj_ty, Type::Generic(_, _)) || obj_ty == Type::StringTy || matches!(obj_ty, Type::Array(_)) {
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                    let get_property = self.emitter.cp.add_method_ref(runtime_class, "getProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;".to_string()).unwrap();
                    let field_idx = self.emitter.cp.add_string(field.clone()).unwrap();
                    self.code.push(Instruction::Ldc_w(field_idx));
                    self.code.push(Instruction::Invokestatic(get_property));
                } else {
                    let class_name = if matches!(&**obj, HirExpr::This(_)) { 
                        self.emitter.class_path.clone().unwrap() 
                    } else if let Type::Class(c) = obj_ty { 
                        match c.as_str() {
                            "Date" => "com/tsdroid/runtime/TsDate".to_string(),
                            "Map" => "com/tsdroid/runtime/TsMap".to_string(),
                            "Set" => "com/tsdroid/runtime/TsSet".to_string(),
                            "RegExp" => "com/tsdroid/runtime/TsRegExp".to_string(),
                            "Error" => "com/tsdroid/runtime/TsError".to_string(),
                            _ => {
                                if c.contains('/') || c.contains('.') {
                                    c.replace('.', "/")
                                } else {
                                    format!("{}/{}", self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0, c)
                                }
                            }
                        }
                    } else { 
                        "java/lang/Object".to_string() 
                    };
                    let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                    let f = self.emitter.cp.add_field_ref(class_idx, field.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Getfield(f));
                }
                
                // Unbox if the final target type is primitive
                self.unbox_if_needed(ty);
                
                // 7. Jump to the end (skip null handler)
                let goto_idx = self.code.len();
                self.code.push(Instruction::Goto(0)); // placeholder
                
                // 8. Null handler target (resolve placeholder Ifnull)
                let null_target_pc = self.code.len();
                self.code[ifnull_idx] = Instruction::Ifnull(null_target_pc as u16);
                
                // Since the stack is empty at this point, we just push aconst_null
                self.code.push(Instruction::Aconst_null);
                
                // 9. Resolve Goto placeholder
                let end_pc = self.code.len();
                self.code[goto_idx] = Instruction::Goto(end_pc as u16);
                
                // 10. Free the temporary slot
                self.emitter.local_slot -= 1;
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn emit_optional_call(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::OptionalCall(callee, args, ty) => {
                // 1. Emit callee
                self.emit_expr(callee);
                let callee_ty = callee.get_type();
                self.box_if_needed(&callee_ty);
                
                // 2. Allocate temporary slot to store the callee
                let callee_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                
                // 3. Store callee in the temporary slot
                self.code.push(Instruction::Astore(callee_slot));
                
                // 4. Load it for the null check
                self.code.push(Instruction::Aload(callee_slot));
                
                // 5. Jump to null handler if the loaded callee is null
                let ifnull_idx = self.code.len();
                self.code.push(Instruction::Ifnull(0)); // placeholder
                
                // 6. Callee is NOT null: evaluate args and invoke
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
                
                // Free temp slots
                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
                
                // 7. Jump to the end (skip null handler)
                let goto_idx = self.code.len();
                self.code.push(Instruction::Goto(0)); // placeholder
                
                // 8. Null handler target (resolve placeholder Ifnull)
                let null_target_pc = self.code.len();
                self.code[ifnull_idx] = Instruction::Ifnull(null_target_pc as u16);
                
                // Since the stack is empty at this point, we just push aconst_null
                self.code.push(Instruction::Aconst_null);
                
                // 9. Resolve Goto placeholder
                let end_pc = self.code.len();
                self.code[goto_idx] = Instruction::Goto(end_pc as u16);
                
                // 10. Free the callee temporary slot
                self.emitter.local_slot -= 1;
            }
            _ => unreachable!(),
        }
    }
}
