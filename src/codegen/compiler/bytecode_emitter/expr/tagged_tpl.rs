use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_tagged_template(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::TaggedTemplate(tag, quasis, exprs) => {
                // 1. Pre-evaluate expressions
                let mut expr_slots = Vec::new();
                for e in exprs {
                    expr_slots.push(self.emit_expr_to_temp(e));
                }

                // 2. Build the quasis ArrayList
                self.code.push(Instruction::New(self.emitter.array_list_class));
                self.code.push(Instruction::Dup);
                self.code.push(Instruction::Invokespecial(self.emitter.array_list_init));
                for q in quasis {
                    self.code.push(Instruction::Dup);
                    self.emit_expr(q);
                    self.code.push(Instruction::Invokevirtual(self.emitter.array_list_add));
                    self.code.push(Instruction::Pop);
                }

                let quasis_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                self.code.push(Instruction::Astore(quasis_slot));

                // 3. Check if tag is a global function
                let is_global_func = if let HirExpr::Var(name, _) = &**tag {
                    !self.emitter.id_registry.contains_key(name)
                } else {
                    false
                };

                let tag_slot = if !is_global_func {
                    Some(self.emit_expr_to_temp(tag))
                } else {
                    None
                };

                // 4. Pack arguments into Object[] (index 0 is quasis, followed by exprs)
                let total_args = 1 + exprs.len();
                if let Some(ts) = tag_slot {
                    self.code.push(Instruction::Aload(ts));
                }
                
                self.code.push(Instruction::Bipush(total_args as i8));
                let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                self.code.push(Instruction::Anewarray(obj_class));

                self.code.push(Instruction::Dup);
                self.code.push(Instruction::Bipush(0));
                self.code.push(Instruction::Aload(quasis_slot));
                self.code.push(Instruction::Aastore);

                for (i, &expr_slot) in expr_slots.iter().enumerate() {
                    self.code.push(Instruction::Dup);
                    self.code.push(Instruction::Bipush((1 + i) as i8));
                    self.code.push(Instruction::Aload(expr_slot));
                    self.code.push(Instruction::Aastore);
                }

                if is_global_func {
                    if let HirExpr::Var(name, _) = &**tag {
                        let app_class_name = if let Some(ref path) = self.emitter.class_path {
                            if path.contains('/') {
                                let package = path.rsplit_once('/').unwrap().0;
                                format!("{}/App", package)
                            } else {
                                "com/tsdroid/app/App".to_string()
                            }
                        } else {
                            "com/tsdroid/app/App".to_string()
                        };
                        let app_class = self.emitter.cp.add_class(&app_class_name).unwrap();
                        let method_ref = self.emitter.cp.add_method_ref(app_class, name.clone(), "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                        self.code.push(Instruction::Invokestatic(method_ref));
                    }
                } else {
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                    let invoke_lambda = self.emitter.cp.add_method_ref(
                        runtime_class,
                        "invokeLambda".to_string(),
                        "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;".to_string(),
                    ).unwrap();
                    self.code.push(Instruction::Invokestatic(invoke_lambda));
                }

                // 6. Free temp slots
                self.emitter.local_slot -= 1; // quasis_slot
                for _ in &expr_slots {
                    self.emitter.local_slot -= 1;
                }
                if tag_slot.is_some() {
                    self.emitter.local_slot -= 1; // tag_slot
                }
            }
            _ => unreachable!(),
        }
    }
}
