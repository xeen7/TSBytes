use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_destructure(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::DestructureObject(fields, rest_opt, source) => {
                // Emit source into a temp slot
                self.emit_expr(source);
                let tmp_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                self.code.push(Instruction::Astore(tmp_slot));
                
                let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                let get_property = self.emitter.cp.add_method_ref(runtime_class, "getProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;".to_string()).unwrap();
                
                for (prop_name, alias, default) in fields {
                    let local_name = alias.as_deref().unwrap_or(prop_name);
                    let slot = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.emitter.id_registry.insert(local_name.to_string(), (slot, Type::Any));
                    self.code.push(Instruction::Aload(tmp_slot));
                    let key_idx = self.emitter.cp.add_string(prop_name.clone()).unwrap();
                    self.code.push(Instruction::Ldc_w(key_idx));
                    self.code.push(Instruction::Invokestatic(get_property));
                    
                    if let Some(def) = default {
                        self.code.push(Instruction::Dup);
                        let else_idx = self.code.len();
                        self.code.push(Instruction::Ifnonnull(0));
                        
                        self.code.push(Instruction::Pop);
                        self.emit_expr(def);
                        let def_ty = self.resolve_type(def);
                        self.box_if_needed(&def_ty);
                        
                        let target_pc = self.code.len();
                        self.code[else_idx] = Instruction::Ifnonnull(target_pc as u16);
                    }
                    self.code.push(Instruction::Astore(slot));
                }
                
                if let Some(rest_name) = rest_opt {
                    let slot = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    self.emitter.id_registry.insert(rest_name.clone(), (slot, Type::Any));
                    
                    let init_m = self.emitter.cp.add_method_ref(ts_object_class, "<init>".to_string(), "()V".to_string()).unwrap();
                    let put_all = self.emitter.cp.add_method_ref(ts_object_class, "putAll".to_string(), "(Ljava/util/Map;)V".to_string()).unwrap();
                    let remove_m = self.emitter.cp.add_method_ref(ts_object_class, "remove".to_string(), "(Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                    
                    // rest = new TsObject()
                    self.code.push(Instruction::New(ts_object_class));
                    self.code.push(Instruction::Dup);
                    self.code.push(Instruction::Invokespecial(init_m));
                    self.code.push(Instruction::Astore(slot));
                    
                    // rest.putAll(source)
                    self.code.push(Instruction::Aload(slot));
                    self.code.push(Instruction::Aload(tmp_slot));
                    self.code.push(Instruction::Checkcast(self.emitter.cp.add_class("java/util/Map").unwrap()));
                    self.code.push(Instruction::Invokevirtual(put_all));
                    
                    // rest.remove(field) for each destructured key
                    for (prop_name, _, _) in fields {
                        self.code.push(Instruction::Aload(slot));
                        let key_idx = self.emitter.cp.add_string(prop_name).unwrap();
                        self.code.push(Instruction::Ldc_w(key_idx));
                        self.code.push(Instruction::Invokevirtual(remove_m));
                        self.code.push(Instruction::Pop);
                    }
                }
            }
            HirStmt::DestructureArray(slots, rest_opt, source) => {
                let source_ty = self.resolve_type(source);
                let elem_ty = match &source_ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::Any,
                };
                
                self.emit_expr(source);
                let tmp_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                self.code.push(Instruction::Astore(tmp_slot));
                
                let al_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                let al_get = self.emitter.cp.add_method_ref(al_class, "get".to_string(), "(I)Ljava/lang/Object;".to_string()).unwrap();
                
                let al_size = self.emitter.cp.add_method_ref(al_class, "size".to_string(), "()I".to_string()).unwrap();
                for (i, (name_opt, default)) in slots.iter().enumerate() {
                    if let Some(name) = name_opt {
                        let slot = self.emitter.local_slot;
                        if elem_ty == Type::Double {
                            self.emitter.local_slot += 2;
                        } else {
                            self.emitter.local_slot += 1;
                        }
                        self.emitter.id_registry.insert(name.clone(), (slot, elem_ty.clone()));
                        
                        // Check if in bounds: i < size
                        self.code.push(Instruction::Aload(tmp_slot));
                        self.code.push(Instruction::Checkcast(al_class));
                        self.code.push(Instruction::Invokevirtual(al_size));
                        self.code.push(Instruction::Bipush(i as i8));
                        let has_elem_idx = self.code.len();
                        self.code.push(Instruction::If_icmpgt(0));
                        
                        // Out of bounds: push null
                        self.code.push(Instruction::Aconst_null);
                        let end_get_idx = self.code.len();
                        self.code.push(Instruction::Goto(0));
                        
                        // In bounds: call get(i)
                        let in_bounds_pc = self.code.len();
                        self.code[has_elem_idx] = Instruction::If_icmpgt(in_bounds_pc as u16);
                        self.code.push(Instruction::Aload(tmp_slot));
                        self.code.push(Instruction::Checkcast(al_class));
                        self.code.push(Instruction::Bipush(i as i8));
                        self.code.push(Instruction::Invokevirtual(al_get));
                        
                        let end_pc = self.code.len();
                        self.code[end_get_idx] = Instruction::Goto(end_pc as u16);
                        
                        // Now we have Object or null on stack. Check if we need default.
                        if let Some(def) = default {
                            self.code.push(Instruction::Dup);
                            let else_idx = self.code.len();
                            self.code.push(Instruction::Ifnonnull(0));
                            
                            self.code.push(Instruction::Pop);
                            self.emit_expr(def);
                            let def_ty = self.resolve_type(def);
                            self.box_if_needed(&def_ty);
                            
                            let target_pc = self.code.len();
                            self.code[else_idx] = Instruction::Ifnonnull(target_pc as u16);
                        }
                        
                        self.unbox_if_needed(&elem_ty);
                        match elem_ty {
                            Type::Double => self.code.push(Instruction::Dstore(slot)),
                            Type::Int | Type::Bool => self.code.push(Instruction::Istore(slot)),
                            _ => self.code.push(Instruction::Astore(slot)),
                        }
                    }
                }
                
                if let Some(rest_name) = rest_opt {
                    let slot = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    let rest_ty = Type::Array(Box::new(elem_ty.clone()));
                    self.emitter.id_registry.insert(rest_name.clone(), (slot, rest_ty));
                    
                    let al_init_col = self.emitter.cp.add_method_ref(al_class, "<init>".to_string(), "(Ljava/util/Collection;)V".to_string()).unwrap();
                    let al_size = self.emitter.cp.add_method_ref(al_class, "size".to_string(), "()I".to_string()).unwrap();
                    let al_sublist = self.emitter.cp.add_method_ref(al_class, "subList".to_string(), "(II)Ljava/util/List;".to_string()).unwrap();
                    
                    let math_class = self.emitter.cp.add_class("java/lang/Math").unwrap();
                    let math_min = self.emitter.cp.add_method_ref(math_class, "min".to_string(), "(II)I".to_string()).unwrap();
                    
                    // rest = new ArrayList(source.subList(min(slots.len(), size), size))
                    self.code.push(Instruction::New(al_class));
                    self.code.push(Instruction::Dup);
                    
                    // Ensure tmp_slot is cast to ArrayList for subList
                    self.code.push(Instruction::Aload(tmp_slot));
                    self.code.push(Instruction::Checkcast(al_class));
                    
                    // fromIndex = min(slots.len(), size)
                    self.code.push(Instruction::Bipush(slots.len() as i8));
                    self.code.push(Instruction::Aload(tmp_slot));
                    self.code.push(Instruction::Checkcast(al_class));
                    self.code.push(Instruction::Invokevirtual(al_size));
                    self.code.push(Instruction::Invokestatic(math_min));
                    
                    // toIndex = size
                    self.code.push(Instruction::Aload(tmp_slot));
                    self.code.push(Instruction::Checkcast(al_class));
                    self.code.push(Instruction::Invokevirtual(al_size));
                    
                    self.code.push(Instruction::Invokevirtual(al_sublist));
                    self.code.push(Instruction::Invokespecial(al_init_col));
                    
                    self.code.push(Instruction::Astore(slot));
                }
            }
            _ => unreachable!(),
        }
    }
}
