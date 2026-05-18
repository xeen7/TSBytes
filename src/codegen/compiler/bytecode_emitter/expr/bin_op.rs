use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_bin_op(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::BinOp(op, left, right, _ty) => {
                            // Handle `"key" in obj` specially — it's not a numeric/string op
                            if matches!(op, BinOp::In) {
                                self.emit_expr(right); // obj (receiver)
                                let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                let has_property = self.emitter.cp.add_method_ref(runtime_class, "hasProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;)Z".to_string()).unwrap();
                                self.emit_expr(left); // key
                                self.box_if_needed(&left.get_type());
                                let string_class = self.emitter.cp.add_class("java/lang/String").unwrap();
                                let value_of = self.emitter.cp.add_method_ref(string_class, "valueOf".to_string(), "(Ljava/lang/Object;)Ljava/lang/String;".to_string()).unwrap();
                                self.code.push(Instruction::Invokestatic(value_of));
                                self.code.push(Instruction::Invokestatic(has_property));
                                return;
                            }
                            
                            // Handle nullish coalescing `left ?? right`
                            if matches!(op, BinOp::NullishCoalesce) {
                                self.emit_expr(left);
                                let left_ty = self.resolve_type(left);
                                self.box_if_needed(&left_ty);
                                
                                let temp_res = self.emitter.local_slot;
                                self.emitter.local_slot += 1;
                                self.code.push(Instruction::Astore(temp_res));
                                
                                self.code.push(Instruction::Aload(temp_res));
                                
                                let right_label_idx = self.code.len();
                                self.code.push(Instruction::Ifnull(0));
                                
                                let end_label_idx = self.code.len();
                                self.code.push(Instruction::Goto(0));
                                
                                let right_pc = self.code.len();
                                self.code[right_label_idx] = Instruction::Ifnull(right_pc as u16);
                                
                                self.emit_expr(right);
                                let right_ty = self.resolve_type(right);
                                self.box_if_needed(&right_ty);
                                
                                self.code.push(Instruction::Astore(temp_res));
                                
                                let end_pc = self.code.len();
                                self.code[end_label_idx] = Instruction::Goto(end_pc as u16);
                                
                                self.code.push(Instruction::Aload(temp_res));
                                self.emitter.local_slot -= 1;
                                return;
                            }
            
                            let left_ty = self.resolve_type(left);
                            let right_ty = self.resolve_type(right);

                            let use_object_equals = matches!(op, BinOp::Eq | BinOp::Ne) && (
                                left_ty == Type::StringTy || right_ty == Type::StringTy 
                                || left_ty == Type::Any || right_ty == Type::Any 
                                || matches!(left_ty, Type::Union(_)) || matches!(right_ty, Type::Union(_))
                            );

                            if use_object_equals {
                                self.emit_expr(left);
                                self.box_if_needed(&left_ty);
                                self.emit_expr(right);
                                self.box_if_needed(&right_ty);
                                
                                let objects_class = self.emitter.cp.add_class("java/util/Objects").unwrap();
                                let objects_equals = self.emitter.cp.add_method_ref(
                                    objects_class,
                                    "equals".to_string(),
                                    "(Ljava/lang/Object;Ljava/lang/Object;)Z".to_string()
                                ).unwrap();
                                self.code.push(Instruction::Invokestatic(objects_equals));
                                
                                if matches!(op, BinOp::Ne) {
                                    self.code.push(Instruction::Iconst_1);
                                    self.code.push(Instruction::Ixor);
                                }
                                return;
                            }
                            let is_bitwise = matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr | BinOp::UShr);
                            let is_exp = matches!(op, BinOp::Exp);

                            // Bitwise ops always operate on int32 — unbox both sides to int
                            if is_bitwise {
                                self.emit_expr(left);
                                self.unbox_to_int(&left_ty);
                                self.emit_expr(right);
                                self.unbox_to_int(&right_ty);
                                match op {
                                    BinOp::BitAnd => self.code.push(Instruction::Iand),
                                    BinOp::BitOr  => self.code.push(Instruction::Ior),
                                    BinOp::BitXor => self.code.push(Instruction::Ixor),
                                    BinOp::Shl    => self.code.push(Instruction::Ishl),
                                    BinOp::Shr    => self.code.push(Instruction::Ishr),
                                    BinOp::UShr   => self.code.push(Instruction::Iushr),
                                    _ => unreachable!(),
                                }
                                // TypeScript numbers are always doubles — promote int result to double
                                self.code.push(Instruction::I2d);
                                return;
                            }

                            // Exponentiation: Math.pow(double, double) -> double
                            if is_exp {
                                let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                                let double_value = self.emitter.cp.add_method_ref(double_class, "doubleValue".to_string(), "()D".to_string()).unwrap();
                                self.emit_expr(left);
                                if left_ty != Type::Double {
                                    if left_ty == Type::Int { self.code.push(Instruction::I2d); }
                                    else { self.box_if_needed(&left_ty); self.code.push(Instruction::Checkcast(double_class)); self.code.push(Instruction::Invokevirtual(double_value)); }
                                }
                                self.emit_expr(right);
                                if right_ty != Type::Double {
                                    if right_ty == Type::Int { self.code.push(Instruction::I2d); }
                                    else { self.box_if_needed(&right_ty); self.code.push(Instruction::Checkcast(double_class)); self.code.push(Instruction::Invokevirtual(double_value)); }
                                }
                                let math_class = self.emitter.cp.add_class("java/lang/Math").unwrap();
                                let pow_method = self.emitter.cp.add_method_ref(math_class, "pow".to_string(), "(DD)D".to_string()).unwrap();
                                self.code.push(Instruction::Invokestatic(pow_method));
                                return;
                            }

                            let is_dynamic = left_ty == Type::Any || right_ty == Type::Any 
                                          || left_ty == Type::StringTy || right_ty == Type::StringTy
                                          || matches!(left_ty, Type::Union(_)) || matches!(right_ty, Type::Union(_));

                            if is_dynamic && matches!(op, BinOp::Add) {
                                self.emit_expr(left);
                                self.box_if_needed(&left_ty);
                                self.emit_expr(right);
                                self.box_if_needed(&right_ty);
                                
                                let tsobject_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                                let add_method = self.emitter.cp.add_method_ref(
                                    tsobject_class, 
                                    "add".to_string(), 
                                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                                ).unwrap();
                                self.code.push(Instruction::Invokestatic(add_method));
                                return;
                            }
            
                            if is_dynamic {
                                // Numeric operation with dynamic types: unbox both to double
                                let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                                let double_value = self.emitter.cp.add_method_ref(double_class, "doubleValue".to_string(), "()D".to_string()).unwrap();
            
                                self.emit_expr(left);
                                if left_ty != Type::Double && left_ty != Type::Int {
                                    self.box_if_needed(&left_ty);
                                    self.code.push(Instruction::Checkcast(double_class));
                                    self.code.push(Instruction::Invokevirtual(double_value));
                                }
            
                                self.emit_expr(right);
                                if right_ty != Type::Double && right_ty != Type::Int {
                                    self.box_if_needed(&right_ty);
                                    self.code.push(Instruction::Checkcast(double_class));
                                    self.code.push(Instruction::Invokevirtual(double_value));
                                }
                            } else {
                                self.emit_expr(left);
                                self.emit_expr(right);
                            }
            
                            // After dynamic numeric unboxing, both sides are now doubles
                            let effective_left_ty = if is_dynamic { Type::Double } else { left_ty };
            
                            match op {
                                BinOp::Add => match effective_left_ty {
                                    Type::Double => self.code.push(Instruction::Dadd),
                                    Type::Int => self.code.push(Instruction::Iadd),
                                    _ => {}
                                },
                                BinOp::Sub => match effective_left_ty {
                                    Type::Double => self.code.push(Instruction::Dsub),
                                    Type::Int => self.code.push(Instruction::Isub),
                                    _ => {}
                                },
                                BinOp::Mul => match effective_left_ty {
                                    Type::Double => self.code.push(Instruction::Dmul),
                                    Type::Int => self.code.push(Instruction::Imul),
                                    _ => {}
                                },
                                BinOp::Div => match effective_left_ty {
                                    Type::Double => self.code.push(Instruction::Ddiv),
                                    Type::Int => self.code.push(Instruction::Idiv),
                                    _ => {}
                                },
                                BinOp::Eq => {
                                    self.code.push(Instruction::Dcmpg);
                                    let ifeq_idx = self.code.len();
                                    self.code.push(Instruction::Ifeq(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[ifeq_idx] = Instruction::Ifeq(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Ne => {
                                    self.code.push(Instruction::Dcmpg);
                                    let ifne_idx = self.code.len();
                                    self.code.push(Instruction::Ifne(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[ifne_idx] = Instruction::Ifne(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Lt => {
                                    self.code.push(Instruction::Dcmpg);
                                    let iflt_idx = self.code.len();
                                    self.code.push(Instruction::Iflt(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[iflt_idx] = Instruction::Iflt(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Le => {
                                    self.code.push(Instruction::Dcmpg);
                                    let ifle_idx = self.code.len();
                                    self.code.push(Instruction::Ifle(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[ifle_idx] = Instruction::Ifle(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Gt => {
                                    self.code.push(Instruction::Dcmpg);
                                    let ifgt_idx = self.code.len();
                                    self.code.push(Instruction::Ifgt(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[ifgt_idx] = Instruction::Ifgt(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Ge => {
                                    self.code.push(Instruction::Dcmpg);
                                    let ifge_idx = self.code.len();
                                    self.code.push(Instruction::Ifge(0));
                                    self.code.push(Instruction::Iconst_0);
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0));
                                    
                                    self.code[ifge_idx] = Instruction::Ifge(self.code.len() as u16);
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    self.code[goto_idx] = Instruction::Goto(self.code.len() as u16);
                                }
                                BinOp::Mod => match effective_left_ty {
                                    Type::Double => self.code.push(Instruction::Drem),
                                    Type::Int => self.code.push(Instruction::Irem),
                                    _ => {}
                                },
                                BinOp::Exp => {
                                    // Math.pow(double, double) -> double
                                    // Both operands already on stack as doubles (via effective_left_ty path)
                                    // but we need to ensure they are doubles, not ints
                                    // Re-emit: we already have them on the stack from the emit above.
                                    // The emitter already pushed both via the is_dynamic / else branch.
                                    let math_class = self.emitter.cp.add_class("java/lang/Math").unwrap();
                                    let pow_method = self.emitter.cp.add_method_ref(
                                        math_class, "pow".to_string(), "(DD)D".to_string()
                                    ).unwrap();
                                    self.code.push(Instruction::Invokestatic(pow_method));
                                }
                                // Bitwise integer ops — operands must be ints
                                BinOp::BitAnd => self.code.push(Instruction::Iand),
                                BinOp::BitOr  => self.code.push(Instruction::Ior),
                                BinOp::BitXor => self.code.push(Instruction::Ixor),
                                BinOp::Shl    => self.code.push(Instruction::Ishl),
                                BinOp::Shr    => self.code.push(Instruction::Ishr),
                                BinOp::UShr   => self.code.push(Instruction::Iushr),
                                _ => {}
                            }
                        }
            _ => unreachable!(),
        }
    }
}
