use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_method_call(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::MethodCall(obj, method_name, args, ty) => {
                if let HirExpr::Var(name, _) = &**obj {
                    if name == "console" && (method_name == "log" || method_name == "error") {
                        if let Some(arg) = args.first() {
                            // 1. Emit the argument first on a completely empty stack
                            self.emit_expr(arg);
                            let arg_ty = self.resolve_type(arg);
                            self.box_if_needed(&arg_ty);
                            
                            // Store in a temporary variable
                            let temp_arg = self.emitter.local_slot;
                            self.emitter.local_slot += 1;
                            self.code.push(Instruction::Astore(temp_arg));
                            
                            // 2. Push System.err or System.out to the stack
                            let sys_class = self.emitter.cp.add_class("java/lang/System").unwrap();
                            let field_name = if method_name == "error" { "err" } else { "out" };
                            let out_field = self.emitter.cp.add_field_ref(sys_class, field_name.to_string(), "Ljava/io/PrintStream;".to_string()).unwrap();
                            self.code.push(Instruction::Getstatic(out_field));
                            
                            // 3. Load the pre-evaluated argument reference
                            self.code.push(Instruction::Aload(temp_arg));
                            
                            // 4. Invoke println
                            let ps_class = self.emitter.cp.add_class("java/io/PrintStream").unwrap();
                            let println_m = self.emitter.cp.add_method_ref(ps_class, "println".to_string(), "(Ljava/lang/Object;)V".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(println_m));
                            
                            self.code.push(Instruction::Aconst_null);
                            
                            // Free the temporary slot
                            self.emitter.local_slot -= 1;
                            return;
                        }
                    }
                }

                // Pre-evaluate receiver obj on an empty stack
                let obj_slot = self.emit_expr_to_temp(obj);
                
                // Pre-evaluate args on an empty stack
                let mut temp_slots = Vec::new();
                for arg in args {
                    temp_slots.push(self.emit_expr_to_temp(arg));
                }

                if let HirExpr::Var(name, _) = &**obj {
                    if name == "Promise" && method_name == "resolve" {
                        if let Some(&temp_slot) = temp_slots.first() {
                            self.code.push(Instruction::Aload(temp_slot));
                        } else {
                            self.code.push(Instruction::Aconst_null);
                        }
                        let cf_class = self.emitter.cp.add_class("java/util/concurrent/CompletableFuture").unwrap();
                        let completed_future = self.emitter.cp.add_method_ref(
                            cf_class,
                            "completedFuture".to_string(),
                            "(Ljava/lang/Object;)Ljava/util/concurrent/CompletableFuture;".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(completed_future));
                        
                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }
                    if name == "String" && method_name == "fromCharCode" {
                        self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                        let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                        self.code.push(Instruction::Anewarray(obj_class));
                        for (i, &temp_slot) in temp_slots.iter().enumerate() {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Bipush(i as i8));
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Aastore);
                        }
                        
                        let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                        let from_char_code_m = self.emitter.cp.add_method_ref(
                            runtime_class,
                            "fromCharCode".to_string(),
                            "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(from_char_code_m));
                        
                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }
                    if name == "Date" {
                        self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                        let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                        self.code.push(Instruction::Anewarray(obj_class));
                        for (i, &temp_slot) in temp_slots.iter().enumerate() {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Bipush(i as i8));
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Aastore);
                        }
                        
                        let date_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsDate").unwrap();
                        let method_idx = self.emitter.cp.add_method_ref(
                            date_class,
                            method_name.clone(),
                            "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(method_idx));
                        self.unbox_if_needed(ty);
                        
                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }
                    if name == "JSON" {
                        self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                        let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                        self.code.push(Instruction::Anewarray(obj_class));
                        for (i, &temp_slot) in temp_slots.iter().enumerate() {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Bipush(i as i8));
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Aastore);
                        }
                        
                        let json_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsJSON").unwrap();
                        let method_idx = self.emitter.cp.add_method_ref(
                            json_class,
                            method_name.clone(),
                            "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(method_idx));
                        self.unbox_if_needed(ty);
                        
                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }
                    if name == "Object" {
                        self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                        let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                        self.code.push(Instruction::Anewarray(obj_class));
                        for (i, &temp_slot) in temp_slots.iter().enumerate() {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Bipush(i as i8));
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Aastore);
                        }
                        
                        let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                        let runtime_method_name = format!("Object_{}", method_name);
                        let method_idx = self.emitter.cp.add_method_ref(
                            runtime_class,
                            runtime_method_name,
                            "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(method_idx));
                        self.unbox_if_needed(ty);
                        
                    }
                    if name == "Number" {
                        self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                        let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                        self.code.push(Instruction::Anewarray(obj_class));
                        for (i, &temp_slot) in temp_slots.iter().enumerate() {
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Bipush(i as i8));
                            self.code.push(Instruction::Aload(temp_slot));
                            self.code.push(Instruction::Aastore);
                        }
                        
                        let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                        let runtime_method_name = format!("Number_{}", method_name);
                        let method_idx = self.emitter.cp.add_method_ref(
                            runtime_class,
                            runtime_method_name,
                            "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Invokestatic(method_idx));
                        self.unbox_if_needed(ty);
                        
                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }
                }

                let obj_ty = obj.get_type();
                let is_queue = match &**obj {
                    HirExpr::Var(name, _) => name == "__queue",
                    HirExpr::FieldGet(_, name, _) => name == "__queue",
                    _ => false,
                } || (if let Type::Class(ref c) = obj_ty { c.contains("LinkedBlockingQueue") } else { false });

                if is_queue && method_name == "put" {
                        self.code.push(Instruction::Aload(obj_slot));
                        let queue_class = self.emitter.cp.add_class("java/util/concurrent/LinkedBlockingQueue").unwrap();
                        self.code.push(Instruction::Checkcast(queue_class));
                        for &temp_slot in &temp_slots {
                            self.code.push(Instruction::Aload(temp_slot));
                        }
                        let put_m = self.emitter.cp.add_method_ref(
                            queue_class,
                            "put".to_string(),
                            "(Ljava/lang/Object;)V".to_string(),
                        ).unwrap();
                        self.code.push(Instruction::Invokevirtual(put_m));
                        self.code.push(Instruction::Aconst_null);

                        for _ in &temp_slots {
                            self.emitter.local_slot -= 1;
                        }
                        self.emitter.local_slot -= 1; // free obj_slot
                        return;
                    }

                // Resolve class from object type for virtual dispatch
                let obj_ty = obj.get_type();
                
                let mut class_name = if matches!(&**obj, HirExpr::This(_)) {
                    self.emitter.class_path.clone().unwrap()
                } else if let Type::Class(ref c) = obj_ty {
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

                if class_name == "java/lang/Object" && method_name == "next" {
                    class_name = "com/tsdroid/runtime/TsGenerator".to_string();
                }

                let builtin_methods = ["trim", "toUpperCase", "toLowerCase", "replace", "split",
                                       "indexOf", "includes", "startsWith", "endsWith",
                                       "substring", "slice", "concat", "repeat", "padStart", "padEnd",
                                       "charAt", "charCodeAt", "trimStart", "trimEnd",
                                       "toFixed", "toPrecision", "toString",
                                       "join", "push", "pop", "shift", "unshift", "map", "filter", "reduce", "forEach", "some", "every", "find"];
                let is_builtin_class = match &obj_ty {
                    Type::Class(ref c) => ["Date", "Map", "Set", "RegExp", "Error", "WeakMap", "WeakSet",
                                           "com/tsdroid/runtime/TsDate", "com/tsdroid/runtime/TsMap", "com/tsdroid/runtime/TsSet", "com/tsdroid/runtime/TsRegExp", "com/tsdroid/runtime/TsError",
                                           "com/tsdroid/runtime/TsWeakMap", "com/tsdroid/runtime/TsWeakSet"].contains(&c.as_str()),
                    _ => false,
                };
                let is_builtin_method = (builtin_methods.contains(&method_name.as_str())
                    && (obj_ty == Type::StringTy || obj_ty == Type::Double || obj_ty == Type::Int || obj_ty == Type::Any || matches!(obj_ty, Type::Array(_))))
                    || is_builtin_class
                    || class_name == "java/lang/Object";
                
                if is_builtin_method {
                    // TsRuntime.callMethod(obj, name, args[]) -> Object
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                    let call_method_m = self.emitter.cp.add_method_ref(
                        runtime_class,
                        "callMethod".to_string(),
                        "(Ljava/lang/Object;Ljava/lang/String;[Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                    ).unwrap();
                    
                    self.code.push(Instruction::Aload(obj_slot));
                    let name_str_idx = self.emitter.cp.add_string(method_name.clone()).unwrap();
                    self.code.push(Instruction::Ldc_w(name_str_idx));
                    self.code.push(Instruction::Bipush(temp_slots.len() as i8));
                    let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                    self.code.push(Instruction::Anewarray(obj_class));
                    for (i, &temp_slot) in temp_slots.iter().enumerate() {
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Bipush(i as i8));
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Aastore);
                    }
                    self.code.push(Instruction::Invokestatic(call_method_m));
                    self.unbox_if_needed(ty);
                    for _ in &temp_slots { self.emitter.local_slot -= 1; }
                    self.emitter.local_slot -= 1;
                    return;
                }

                // Load receiver obj
                self.code.push(Instruction::Aload(obj_slot));
                if class_name == "com/tsdroid/runtime/TsGenerator" {
                    let ts_gen_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsGenerator").unwrap();
                    self.code.push(Instruction::Checkcast(ts_gen_class));
                }
                
                let has_spread = args.iter().any(|a| matches!(a, HirExpr::Spread(_)));
                if has_spread {
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
                    let array_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    let to_array = self.emitter.cp.add_method_ref(array_class, "toArray".to_string(), "()[Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(to_array));
                } else {
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
                
                let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                let method_idx = self.emitter.cp.add_method_ref(class_idx, method_name.clone(), "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                self.code.push(Instruction::Invokevirtual(method_idx));
                self.unbox_if_needed(ty);

                // Free temp slots
                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
                self.emitter.local_slot -= 1; // free obj_slot
            }
            _ => unreachable!(),
        }
    }
}
