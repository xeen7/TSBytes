use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_index(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::IndexGet(obj, idx, ty) => {
                let obj_ty = self.resolve_type(obj);
                
                if let Type::Array(inner) = obj_ty {
                    self.emit_expr(obj);
                    let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    self.code.push(Instruction::Checkcast(array_list_class)); // Guarantee type for verifier
                    
                    self.emit_expr(idx);
                    
                    let idx_ty = self.resolve_type(idx);
                    if idx_ty == Type::Double || idx_ty == Type::Any {
                        self.box_if_needed(&idx_ty);
                        let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                        let int_value = self.emitter.cp.add_method_ref(double_class, "intValue".to_string(), "()I".to_string()).unwrap();
                        self.code.push(Instruction::Checkcast(double_class));
                        self.code.push(Instruction::Invokevirtual(int_value));
                    }
                    
                    let get = self.emitter.cp.add_method_ref(array_list_class, "get".to_string(), "(I)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(get));
                    self.unbox_if_needed(inner.as_ref());
                } else {
                    // obj_ty is Any or Object: perform a dynamic runtime check for List vs Map
                    self.emit_expr(obj);
                    
                    let temp_obj = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_obj));
                    
                    self.emit_expr(idx);
                    let idx_ty = self.resolve_type(idx);
                    self.box_if_needed(&idx_ty); // Box index first to ensure it's a reference type on the stack
                    
                    let temp_idx = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_idx));
                    
                    // 1. Check if the object is an instance of java/util/List
                    self.code.push(Instruction::Aload(temp_obj));
                    let list_class = self.emitter.cp.add_class("java/util/List").unwrap();
                    self.code.push(Instruction::Instanceof(list_class));
                    
                    let map_label_idx = self.code.len();
                    self.code.push(Instruction::Ifeq(0)); // placeholder, branch to map access if false
                    
                    // --- List Access Path ---
                    self.code.push(Instruction::Aload(temp_obj));
                    let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    self.code.push(Instruction::Checkcast(array_list_class));
                    self.code.push(Instruction::Aload(temp_idx));
                    
                    // Convert double/object index to integer
                    let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                    let int_value = self.emitter.cp.add_method_ref(double_class, "intValue".to_string(), "()I".to_string()).unwrap();
                    self.code.push(Instruction::Checkcast(double_class));
                    self.code.push(Instruction::Invokevirtual(int_value));
                    
                    let get = self.emitter.cp.add_method_ref(array_list_class, "get".to_string(), "(I)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(get));
                    
                    // Reuse temp_obj to store the result to keep stack empty at branch merge point
                    self.code.push(Instruction::Astore(temp_obj));
                    
                    let end_label_idx = self.code.len();
                    self.code.push(Instruction::Goto(0)); // placeholder to skip map access
                    
                    // --- Map Access Path ---
                    let map_label_pc = self.code.len();
                    self.code[map_label_idx] = Instruction::Ifeq(map_label_pc as u16);
                    
                    self.code.push(Instruction::Aload(temp_obj));
                    self.code.push(Instruction::Aload(temp_idx)); // Index is already boxed
                    
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                    let map_get = self.emitter.cp.add_method_ref(runtime_class, "mapGet".to_string(), "(Ljava/util/Map;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokestatic(map_get));
                    
                    // Reuse temp_obj to store the result
                    self.code.push(Instruction::Astore(temp_obj));
                    
                    // --- End ---
                    let end_pc = self.code.len();
                    self.code[end_label_idx] = Instruction::Goto(end_pc as u16);
                    
                    // Load the result back to stack
                    self.code.push(Instruction::Aload(temp_obj));
                    self.unbox_if_needed(ty);
                    
                    // Free temp slots
                    self.emitter.local_slot -= 2;
                }
            }
            HirExpr::IndexSet(obj, idx, val, _ty) => {
                let obj_ty = self.resolve_type(obj);
                
                if let Type::Array(_) = obj_ty {
                    self.emit_expr(obj);
                    self.emit_expr(idx);
                    
                    let idx_ty = self.resolve_type(idx);
                    if idx_ty == Type::Double || idx_ty == Type::Any {
                        self.box_if_needed(&idx_ty);
                        let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                        let int_value = self.emitter.cp.add_method_ref(double_class, "intValue".to_string(), "()I".to_string()).unwrap();
                        self.code.push(Instruction::Checkcast(double_class));
                        self.code.push(Instruction::Invokevirtual(int_value));
                    }
                    
                    self.emit_expr(val);
                    let val_ty = self.resolve_type(val);
                    self.box_if_needed(&val_ty);
                    
                    // Store the boxed new value in a temporary slot to avoid stack alignment issues
                    let temp_val = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_val));
                    
                    // Load it to be consumed by ArrayList.set
                    self.code.push(Instruction::Aload(temp_val));
                    
                    let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    let set = self.emitter.cp.add_method_ref(array_list_class, "set".to_string(), "(ILjava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(set));
                    self.code.push(Instruction::Pop); // Discard old value returned by set
                    
                    // Load the new value to leave it on the stack as boxed Object
                    self.code.push(Instruction::Aload(temp_val));
                    
                    self.emitter.local_slot -= 1;
                } else {
                    // obj_ty is Any or Object: perform a dynamic runtime check for List vs Map
                    self.emit_expr(obj);
                    let temp_obj = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_obj));
                    
                    self.emit_expr(idx);
                    let idx_ty = self.resolve_type(idx);
                    self.box_if_needed(&idx_ty); // Box index first to ensure it's a reference type
                    
                    let temp_idx = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_idx));
                    
                    self.emit_expr(val);
                    let val_ty = self.resolve_type(val);
                    self.box_if_needed(&val_ty); // Box value first to ensure it's a reference type
                    
                    let temp_val = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.code.push(Instruction::Astore(temp_val));
                    
                    // 1. Check if the object is an instance of java/util/List
                    self.code.push(Instruction::Aload(temp_obj));
                    let list_class = self.emitter.cp.add_class("java/util/List").unwrap();
                    self.code.push(Instruction::Instanceof(list_class));
                    
                    let map_label_idx = self.code.len();
                    self.code.push(Instruction::Ifeq(0)); // placeholder, branch to map access if false
                    
                    // --- List Access Path ---
                    self.code.push(Instruction::Aload(temp_obj));
                    let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                    self.code.push(Instruction::Checkcast(array_list_class));
                    self.code.push(Instruction::Aload(temp_idx));
                    
                    // Convert double/object index to integer
                    let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                    let int_value = self.emitter.cp.add_method_ref(double_class, "intValue".to_string(), "()I".to_string()).unwrap();
                    self.code.push(Instruction::Checkcast(double_class));
                    self.code.push(Instruction::Invokevirtual(int_value));
                    
                    self.code.push(Instruction::Aload(temp_val)); // Value is already boxed
                    
                    let set = self.emitter.cp.add_method_ref(array_list_class, "set".to_string(), "(ILjava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokevirtual(set));
                    self.code.push(Instruction::Pop); // Discard old value returned by set
                    
                    let end_label_idx = self.code.len();
                    self.code.push(Instruction::Goto(0)); // placeholder to skip map access
                    
                    // --- Map Access Path ---
                    let map_label_pc = self.code.len();
                    self.code[map_label_idx] = Instruction::Ifeq(map_label_pc as u16);
                    
                    self.code.push(Instruction::Aload(temp_obj));
                    self.code.push(Instruction::Aload(temp_idx)); // Index is already boxed
                    self.code.push(Instruction::Aload(temp_val)); // Value is already boxed
                    
                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                    let map_put = self.emitter.cp.add_method_ref(runtime_class, "mapPut".to_string(), "(Ljava/util/Map;Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                    self.code.push(Instruction::Invokestatic(map_put));
                    self.code.push(Instruction::Pop); // Discard old value
                    
                    // --- End ---
                    let end_pc = self.code.len();
                    self.code[end_label_idx] = Instruction::Goto(end_pc as u16);
                    
                    // Load the assigned value back to stack to leave it on the stack as boxed Object
                    self.code.push(Instruction::Aload(temp_val));
                    
                    // Free temp slots
                    self.emitter.local_slot -= 3;
                }
            }
            _ => unreachable!(),
        }
    }
}
