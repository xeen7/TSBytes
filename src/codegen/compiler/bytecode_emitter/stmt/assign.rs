use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_assign(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Assign(name, expr) => {
                            self.emit_expr(expr);
                            let expr_ty = self.resolve_type(expr);
                            if let Some((slot, var_ty)) = self.emitter.id_registry.get(name).cloned() {
                                if var_ty == Type::Any && expr_ty.is_primitive() {
                                    self.box_if_needed(&expr_ty);
                                } else if var_ty.is_primitive() && expr_ty == Type::Any {
                                    self.unbox_if_needed(&var_ty);
                                }
                                
                                match var_ty {
                                    Type::Double => self.code.push(Instruction::Dstore(slot)),
                                    Type::Int => self.code.push(Instruction::Istore(slot)),
                                    Type::Bool => self.code.push(Instruction::Istore(slot)),
                                    _ => self.code.push(Instruction::Astore(slot)),
                                }
                            } else {
                                match expr_ty {
                                    Type::Double => self.code.push(Instruction::Pop2),
                                    Type::Void => {},
                                    _ => self.code.push(Instruction::Pop),
                                }
                            }
                        }
            HirStmt::CompoundAssign(name, op, expr) => {
                            if let Some((slot, var_ty)) = self.emitter.id_registry.get(name).cloned() {
                                if *op == AssignOp::AddAssign && (var_ty == Type::StringTy || var_ty == Type::Any) {
                                    // Delegate to emit_bin_op logic for robust string concatenation
                                    let add_expr = HirExpr::BinOp(BinOp::Add, Box::new(HirExpr::Var(name.clone(), var_ty.clone())), Box::new(expr.clone()), Type::Any);
                                    self.emit_expr(&add_expr);
                                    let add_ty = self.resolve_type(&add_expr);
                                    
                                    if var_ty == Type::Any && add_ty.is_primitive() {
                                        self.box_if_needed(&add_ty);
                                    } else if var_ty.is_primitive() && add_ty == Type::Any {
                                        self.unbox_if_needed(&var_ty);
                                    }
                                    
                                    match var_ty {
                                        Type::Double => self.code.push(Instruction::Dstore(slot)),
                                        Type::Int | Type::Bool => self.code.push(Instruction::Istore(slot)),
                                        _ => self.code.push(Instruction::Astore(slot)),
                                    }
                                    return;
                                }

                                // Load current value of the variable onto the stack
                                let use_int = matches!(op,
                                    AssignOp::BitAndAssign | AssignOp::BitOrAssign | AssignOp::BitXorAssign
                                    | AssignOp::ShlAssign | AssignOp::ShrAssign | AssignOp::UShrAssign
                                );

                                if use_int {
                                    // Bitwise: load + unbox to int
                                    match var_ty {
                                        Type::Int => self.code.push(Instruction::Iload(slot)),
                                        Type::Double => { self.code.push(Instruction::Dload(slot)); self.code.push(Instruction::D2i); }
                                        _ => { self.code.push(Instruction::Aload(slot)); self.unbox_to_int(&var_ty); }
                                    }
                                    self.emit_expr(expr);
                                    let expr_ty = self.resolve_type(expr);
                                    self.unbox_to_int(&expr_ty);
                                    match op {
                                        AssignOp::BitAndAssign => self.code.push(Instruction::Iand),
                                        AssignOp::BitOrAssign  => self.code.push(Instruction::Ior),
                                        AssignOp::BitXorAssign => self.code.push(Instruction::Ixor),
                                        AssignOp::ShlAssign    => self.code.push(Instruction::Ishl),
                                        AssignOp::ShrAssign    => self.code.push(Instruction::Ishr),
                                        AssignOp::UShrAssign   => self.code.push(Instruction::Iushr),
                                        _ => unreachable!(),
                                    }
                                    // Store result back
                                    match var_ty {
                                        Type::Int => self.code.push(Instruction::Istore(slot)),
                                        Type::Any => {
                                            self.code.push(Instruction::I2d);
                                            self.code.push(Instruction::Invokestatic(self.emitter.double_value_of));
                                            self.code.push(Instruction::Astore(slot));
                                        }
                                        _ => self.code.push(Instruction::Istore(slot)),
                                    }
                                } else if matches!(op, AssignOp::LogAndAssign | AssignOp::LogOrAssign | AssignOp::NullishAssign) {
                                    // Logical/nullish assignment: short-circuit store
                                    self.code.push(Instruction::Aload(slot));
                                    let check_idx = self.code.len();
                                    match op {
                                        AssignOp::LogAndAssign  => self.code.push(Instruction::Ifnull(0)),   // if null/false, skip
                                        AssignOp::LogOrAssign   => self.code.push(Instruction::Ifnonnull(0)), // if truthy, skip
                                        AssignOp::NullishAssign => self.code.push(Instruction::Ifnonnull(0)), // if non-null, skip
                                        _ => unreachable!(),
                                    }
                                    self.emit_expr(expr);
                                    let expr_ty = self.resolve_type(expr);
                                    self.box_if_needed(&expr_ty);
                                    self.code.push(Instruction::Astore(slot));
                                    self.code[check_idx] = match op {
                                        AssignOp::LogAndAssign  => Instruction::Ifnull(self.code.len() as u16),
                                        _                        => Instruction::Ifnonnull(self.code.len() as u16),
                                    };
                                } else {
                                    // Numeric compound ops: load as double
                                    if var_ty == Type::Any {
                                        self.code.push(Instruction::Aload(slot));
                                        self.unbox_if_needed(&Type::Double);
                                    } else if var_ty == Type::Int {
                                        self.code.push(Instruction::Iload(slot));
                                        self.code.push(Instruction::I2d);
                                    } else {
                                        self.code.push(Instruction::Dload(slot));
                                    }

                                    self.emit_expr(expr);
                                    let expr_ty = self.resolve_type(expr);
                                    if expr_ty != Type::Double {
                                        if expr_ty == Type::Int { self.code.push(Instruction::I2d); }
                                        else { self.unbox_if_needed(&Type::Double); }
                                    }

                                    match op {
                                        AssignOp::AddAssign => self.code.push(Instruction::Dadd),
                                        AssignOp::SubAssign => self.code.push(Instruction::Dsub),
                                        AssignOp::MulAssign => self.code.push(Instruction::Dmul),
                                        AssignOp::DivAssign => self.code.push(Instruction::Ddiv),
                                        AssignOp::ModAssign => self.code.push(Instruction::Drem),
                                        AssignOp::ExpAssign => {
                                            let math_class = self.emitter.cp.add_class("java/lang/Math").unwrap();
                                            let pow = self.emitter.cp.add_method_ref(math_class, "pow".to_string(), "(DD)D".to_string()).unwrap();
                                            self.code.push(Instruction::Invokestatic(pow));
                                        }
                                        _ => self.code.push(Instruction::Dadd),
                                    }

                                    if var_ty == Type::Any {
                                        self.code.push(Instruction::Invokestatic(self.emitter.double_value_of));
                                        self.code.push(Instruction::Astore(slot));
                                    } else if var_ty == Type::Int {
                                        self.code.push(Instruction::D2i);
                                        self.code.push(Instruction::Istore(slot));
                                    } else {
                                        self.code.push(Instruction::Dstore(slot));
                                    }
                                }
                            }
                        }

             HirStmt::FieldAssign(obj, field, val) => {
                            let mut is_static = false;
                            if let HirExpr::Var(ref name, _) = obj {
                                if !self.emitter.id_registry.contains_key(name) {
                                    is_static = true;
                                }
                            }
                            
                            if is_static {
                                let class_name = if let HirExpr::Var(ref name, _) = obj {
                                    let package_name = self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0;
                                    format!("{}/{}", package_name, name)
                                } else {
                                    "java/lang/Object".to_string()
                                };
                                self.emit_expr(&val);
                                let val_ty = self.resolve_type(val);
                                self.box_if_needed(&val_ty);
                                
                                let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                let field_idx = self.emitter.cp.add_field_ref(class_idx, field.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                                self.code.push(Instruction::Putstatic(field_idx));
                            } else {
                                let obj_ty = self.resolve_type(&obj);
                                let is_builtin_class = match &obj_ty {
                                    Type::Class(ref c) => ["Date", "Map", "Set", "RegExp", "Error", "com/tsdroid/runtime/TsDate", "com/tsdroid/runtime/TsMap", "com/tsdroid/runtime/TsSet", "com/tsdroid/runtime/TsRegExp", "com/tsdroid/runtime/TsError"].contains(&c.as_str()),
                                    _ => false,
                                };
                                if is_builtin_class || matches!(obj_ty, Type::Object(_)) || obj_ty == Type::Any || matches!(obj_ty, Type::Generic(_, _)) || obj_ty == Type::StringTy || matches!(obj_ty, Type::Array(_)) {
                                    self.emit_expr(&obj);
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let set_property = self.emitter.cp.add_method_ref(runtime_class, "setProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
                                    let field_idx = self.emitter.cp.add_string(field.clone()).unwrap();
                                    self.code.push(Instruction::Ldc_w(field_idx));
                                    self.emit_expr(&val);
                                    let val_ty = self.resolve_type(val);
                                    self.box_if_needed(&val_ty);
                                    self.code.push(Instruction::Invokestatic(set_property));
                                } else {
                                    self.emit_expr(&obj);
                                    self.emit_expr(&val);
                                    let val_ty = self.resolve_type(val);
                                    self.box_if_needed(&val_ty);
                                    let raw_c = match obj_ty {
                                        Type::Class(ref c) => c.rsplit('/').next().unwrap().to_string(),
                                        _ if matches!(&obj, HirExpr::This(_)) => self.emitter.class_path.as_ref().unwrap().rsplit('/').next().unwrap().to_string(),
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
                                    } else if matches!(&obj, HirExpr::This(_)) { 
                                        self.emitter.class_path.clone().unwrap() 
                                    } else { 
                                        "java/lang/Object".to_string() 
                                    };
                                    let class_idx = self.emitter.cp.add_class(&class_name).unwrap();
                                    let is_setter = if let Some(class_def) = self.emitter.all_classes.get(&raw_c) {
                                        class_def.members.iter().any(|m| matches!(m, ClassMember::Setter(name, ..) if name == field))
                                    } else { false };

                                    if is_setter {
                                        let method_idx = self.emitter.cp.add_method_ref(class_idx, format!("set${}", field), "(Ljava/lang/Object;)V".to_string()).unwrap();
                                        self.code.push(Instruction::Invokevirtual(method_idx));
                                    } else {
                                        let field_idx = self.emitter.cp.add_field_ref(class_idx, field.clone(), "Ljava/lang/Object;".to_string()).unwrap();
                                        self.code.push(Instruction::Putfield(field_idx));
                                    }
                                }
                            }
                        }
            _ => unreachable!(),
        }
    }
}
