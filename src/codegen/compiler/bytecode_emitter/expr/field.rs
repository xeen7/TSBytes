use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_field(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::FieldGet(obj, field_name, ty) => {
                            let mut is_static = false;
                            if let HirExpr::Var(ref name, _) = &**obj {
                                if !self.emitter.id_registry.contains_key(name) {
                                    is_static = true;
                                }
                            }
                            
                            if is_static {
                                let (class_name, final_field_name) = if let HirExpr::Var(ref name, _) = &**obj {
                                    if name == "Number" {
                                        ("com/tsdroid/runtime/TsRuntime".to_string(), format!("Number_{}", field_name))
                                    } else if name == "Math" {
                                        ("com/tsdroid/runtime/TsRuntime".to_string(), format!("Math_{}", field_name))
                                    } else {
                                        let package_name = self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0;
                                        (format!("{}/{}", package_name, name), field_name.clone())
                                    }
                                } else {
                                    ("java/lang/Object".to_string(), field_name.clone())
                                };
                                let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                let field_idx = self.emitter.cp.add_field_ref(class_idx, final_field_name, "Ljava/lang/Object;".to_string()).unwrap();
                                self.code.push(Instruction::Getstatic(field_idx));
                                self.unbox_if_needed(ty);
                            } else {
                                self.emit_expr(obj);
                                let obj_ty = self.resolve_type(obj);
                                let is_builtin_class = match &obj_ty {
                                    Type::Class(ref c) => ["Date", "Map", "Set", "RegExp", "Error", "com/tsdroid/runtime/TsDate", "com/tsdroid/runtime/TsMap", "com/tsdroid/runtime/TsSet", "com/tsdroid/runtime/TsRegExp", "com/tsdroid/runtime/TsError"].contains(&c.as_str()),
                                    _ => false,
                                };
                                if is_builtin_class || matches!(obj_ty, Type::Object(_)) || obj_ty == Type::Any || matches!(obj_ty, Type::Generic(_, _)) || obj_ty == Type::StringTy || matches!(obj_ty, Type::Array(_)) {
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let get_property = self.emitter.cp.add_method_ref(runtime_class, "getProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;".to_string()).unwrap();
                                    let field_idx = self.emitter.cp.add_string(field_name.clone()).unwrap();
                                    self.code.push(Instruction::Ldc_w(field_idx));
                                    self.code.push(Instruction::Invokestatic(get_property));
                                    self.unbox_if_needed(ty);
                                } else {
                                    let raw_c = match obj_ty {
                                        Type::Class(ref c) => c.rsplit('/').next().unwrap().to_string(),
                                        _ if matches!(&**obj, HirExpr::This(_)) => self.emitter.class_path.as_ref().unwrap().rsplit('/').next().unwrap().to_string(),
                                        _ => "".to_string(),
                                    };
                                    let class_name = if let Type::Class(c) = &obj_ty { 
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
                                    } else if matches!(&**obj, HirExpr::This(_)) { 
                                        self.emitter.class_path.clone().unwrap() 
                                    } else { 
                                        "java/lang/Object".to_string() 
                                    };
                                    let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                    let is_getter = if let Some(class_def) = self.emitter.all_classes.get(&raw_c) {
                                        class_def.members.iter().any(|m| matches!(m, ClassMember::Getter(name, ..) if name == field_name))
                                    } else { false };

                                    if is_getter {
                                        let method_idx = self.emitter.cp.add_method_ref(class_idx, format!("get${}", field_name), "()Ljava/lang/Object;".to_string()).unwrap();
                                        self.code.push(Instruction::Invokevirtual(method_idx));
                                        self.unbox_if_needed(ty);
                                    } else {
                                        let field_idx = self.emitter.cp.add_field_ref(class_idx, field_name.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                                        self.code.push(Instruction::Getfield(field_idx));
                                        self.unbox_if_needed(ty);
                                    }
                                }
                            }
                        }
            HirExpr::FieldSet(obj, field_name, val, ty) => {
                            let mut is_static = false;
                            if let HirExpr::Var(ref name, _) = &**obj {
                                if !self.emitter.id_registry.contains_key(name) {
                                    is_static = true;
                                }
                            }
                            
                            if is_static {
                                let class_name = if let HirExpr::Var(ref name, _) = &**obj {
                                    let package_name = self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0;
                                    format!("{}/{}", package_name, name)
                                } else {
                                    "java/lang/Object".to_string()
                                };
                                self.emit_expr(val);
                                self.box_if_needed(ty);
                                
                                let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                let field_idx = self.emitter.cp.add_field_ref(class_idx, field_name.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                                self.code.push(Instruction::Putstatic(field_idx));
                            } else {
                                let obj_ty = self.resolve_type(obj);
                                let is_builtin_class = match &obj_ty {
                                    Type::Class(ref c) => ["Date", "Map", "Set", "RegExp", "Error", "com/tsdroid/runtime/TsDate", "com/tsdroid/runtime/TsMap", "com/tsdroid/runtime/TsSet", "com/tsdroid/runtime/TsRegExp", "com/tsdroid/runtime/TsError"].contains(&c.as_str()),
                                    _ => false,
                                };
                                if is_builtin_class || matches!(obj_ty, Type::Object(_)) || obj_ty == Type::Any || matches!(obj_ty, Type::Generic(_, _)) || obj_ty == Type::StringTy || matches!(obj_ty, Type::Array(_)) {
                                    self.emit_expr(obj);
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let set_property = self.emitter.cp.add_method_ref(runtime_class, "setProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
                                    let field_idx = self.emitter.cp.add_string(field_name.clone()).unwrap();
                                    self.code.push(Instruction::Ldc_w(field_idx));
                                    self.emit_expr(val);
                                    self.box_if_needed(ty);
                                    self.code.push(Instruction::Invokestatic(set_property));
                                } else {
                                    self.emit_expr(obj);
                                    self.emit_expr(val);
                                    self.box_if_needed(ty);
                                    let raw_c = match obj_ty {
                                        Type::Class(ref c) => c.rsplit('/').next().unwrap().to_string(),
                                        _ if matches!(&**obj, HirExpr::This(_)) => self.emitter.class_path.as_ref().unwrap().rsplit('/').next().unwrap().to_string(),
                                        _ => "".to_string(),
                                    };
                                    let class_name = if let Type::Class(c) = &obj_ty { 
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
                                    } else if matches!(&**obj, HirExpr::This(_)) { 
                                        self.emitter.class_path.clone().unwrap() 
                                    } else { 
                                        "java/lang/Object".to_string() 
                                    };
                                    let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                    let is_setter = if let Some(class_def) = self.emitter.all_classes.get(&raw_c) {
                                        class_def.members.iter().any(|m| matches!(m, ClassMember::Setter(name, ..) if name == field_name))
                                    } else { false };

                                    if is_setter {
                                        let method_idx = self.emitter.cp.add_method_ref(class_idx, format!("set${}", field_name), "(Ljava/lang/Object;)V".to_string()).unwrap();
                                        self.code.push(Instruction::Invokevirtual(method_idx));
                                    } else {
                                        let field_idx = self.emitter.cp.add_field_ref(class_idx, field_name.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                                        self.code.push(Instruction::Putfield(field_idx));
                                    }
                                }
                            }
                        }
            _ => unreachable!(),
        }
    }
}
