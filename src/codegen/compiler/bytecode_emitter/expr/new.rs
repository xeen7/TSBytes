use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_new(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::New(class_name, args, _ty) => {
                let is_java_or_runtime_class = class_name.contains('/') || class_name.contains('.') || class_name == "Object";
                let full_class_name = match class_name.as_str() {
                    "Error" => "com/tsdroid/runtime/TsError".to_string(),
                    "Date" => "com/tsdroid/runtime/TsDate".to_string(),
                    "Map" => "com/tsdroid/runtime/TsMap".to_string(),
                    "Set" => "com/tsdroid/runtime/TsSet".to_string(),
                    "RegExp" => "com/tsdroid/runtime/TsRegExp".to_string(),
                    "Object" => "com/tsdroid/runtime/TsObject".to_string(),
                    "WeakMap" => "com/tsdroid/runtime/TsWeakMap".to_string(),
                    "WeakSet" => "com/tsdroid/runtime/TsWeakSet".to_string(),
                    _ => {
                        if class_name.contains('/') {
                            class_name.clone()
                        } else if class_name.contains('.') {
                            class_name.replace('.', "/")
                        } else {
                            let package_name = self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0;
                            format!("{}/{}", package_name, class_name)
                        }
                    }
                };
                let class_idx = self.emitter.cp.add_class(&full_class_name).unwrap();
                self.code.push(Instruction::New(class_idx));
                self.code.push(Instruction::Dup);

                if is_java_or_runtime_class {
                    // For Java and runtime classes, evaluate each argument and push it directly to the stack
                    let mut arg_types = Vec::new();
                    for arg in args {
                        self.emit_expr(arg);
                        let ty = self.resolve_type(arg);
                        self.box_if_needed(&ty);
                        arg_types.push(ty);
                    }
                    
                    // Build the constructor descriptor: (arg_types...)V
                    let mut descriptor = "(".to_string();
                    for ty in arg_types {
                        match ty {
                            Type::Class(ref c) => {
                                descriptor.push_str(&format!("L{};", c.replace('.', "/")));
                            }
                            _ => {
                                descriptor.push_str("Ljava/lang/Object;");
                            }
                        }
                    }
                    descriptor.push_str(")V");
                    
                    let init_idx = self.emitter.cp.add_method_ref(class_idx, "<init>", &descriptor).unwrap();
                    self.code.push(Instruction::Invokespecial(init_idx));
                } else {
                    // Pre-evaluate args on an empty stack using temporary slots
                    let mut temp_slots = Vec::new();
                    for arg in args {
                        temp_slots.push(self.emit_expr_to_temp(arg));
                    }

                    // Build Object[] to match the constructor descriptor ([Ljava/lang/Object;)V
                    self.code.push(Instruction::Bipush(args.len() as i8));
                    let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                    self.code.push(Instruction::Anewarray(obj_class));
                    for (i, &temp_slot) in temp_slots.iter().enumerate() {
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Bipush(i as i8));
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Aastore);
                    }

                    let init_idx = self.emitter.cp.add_method_ref(class_idx, "<init>", "([Ljava/lang/Object;)V").unwrap();
                    self.code.push(Instruction::Invokespecial(init_idx));

                    // Free temp slots
                    for _ in &temp_slots {
                        self.emitter.local_slot -= 1;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
