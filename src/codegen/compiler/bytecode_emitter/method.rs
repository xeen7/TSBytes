use ristretto_classfile::attributes::{Instruction, CodeException};
use crate::codegen::compiler::ir::*;
use super::BytecodeEmitter;

pub(crate) struct MethodCodeGen<'a> {
    pub(crate) code: Vec<Instruction>,
    pub(crate) emitter: &'a mut BytecodeEmitter,
    pub(crate) loop_stack: Vec<(Option<String>, usize, Vec<usize>, Vec<usize>)>, // (label, start_idx, break_indices, continue_indices)
    pub(crate) exceptions: Vec<CodeException>,
    pub(crate) is_async: bool,
    pub(crate) pending_label: Option<String>,
}

pub(crate) fn inst_size(inst: &Instruction) -> usize {
    match inst {
        // 2-byte instructions (opcode + 1 operand byte)
        Instruction::Bipush(_) |
        Instruction::Aload(_) | Instruction::Astore(_) |
        Instruction::Iload(_) | Instruction::Istore(_) |
        Instruction::Dload(_) | Instruction::Dstore(_) |
        Instruction::Fload(_) | Instruction::Fstore(_) |
        Instruction::Lload(_) | Instruction::Lstore(_) |
        Instruction::Newarray(_) => 2,
        // 3-byte instructions (opcode + 2 operand bytes)
        Instruction::Sipush(_) |
        Instruction::Ldc_w(_) | Instruction::Ldc2_w(_) |
        Instruction::Ifeq(_) | Instruction::Ifne(_) |
        Instruction::Iflt(_) | Instruction::Ifle(_) |
        Instruction::Ifgt(_) | Instruction::Ifge(_) |
        Instruction::Ifnull(_) | Instruction::Ifnonnull(_) |
        Instruction::If_icmpeq(_) | Instruction::If_icmpne(_) |
        Instruction::If_icmplt(_) | Instruction::If_icmple(_) |
        Instruction::If_icmpgt(_) | Instruction::If_icmpge(_) |
        Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) |
        Instruction::Goto(_) |
        Instruction::Invokespecial(_) | Instruction::Invokevirtual(_) |
        Instruction::Invokestatic(_) | Instruction::New(_) |
        Instruction::Checkcast(_) | Instruction::Instanceof(_) |
        Instruction::Putfield(_) | Instruction::Getfield(_) |
        Instruction::Putstatic(_) | Instruction::Getstatic(_) |
        Instruction::Anewarray(_) | Instruction::Iinc(_, _) => 3,
        // 5-byte instructions
        Instruction::Invokeinterface(_, _) => 5,
        // 1-byte instructions (everything else)
        _ => 1,
    }
}

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn resolve_type(&self, expr: &HirExpr) -> Type {
        match expr {
            HirExpr::Var(name, ty) => {
                if let Some((_, true_ty)) = self.emitter.id_registry.get(name) {
                    true_ty.clone()
                } else {
                    ty.clone()
                }
            }
            HirExpr::BinOp(op, left, right, _ty) => {
                let left_ty = self.resolve_type(left);
                let right_ty = self.resolve_type(right);
                let is_dynamic = left_ty == Type::Any || right_ty == Type::Any 
                              || left_ty == Type::StringTy || right_ty == Type::StringTy
                              || matches!(left_ty, Type::Union(_)) || matches!(right_ty, Type::Union(_));
                match op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::In => Type::Bool,
                    BinOp::Add if is_dynamic => Type::Any,
                    BinOp::Mul | BinOp::Sub | BinOp::Div | BinOp::Mod if is_dynamic => Type::Double,
                    _ => {
                        if left_ty == Type::Double || right_ty == Type::Double { Type::Double }
                        else if left_ty == Type::Int && right_ty == Type::Int { Type::Int }
                        else { Type::Any }
                    }
                }
            }
            HirExpr::IndexGet(obj, _idx, ty) => {
                let obj_ty = self.resolve_type(obj);
                if let Type::Array(inner) = obj_ty {
                    (*inner).clone()
                } else {
                    ty.clone()
                }
            }
            HirExpr::FieldSet(..) => Type::Void,
            HirExpr::IndexSet(..) => Type::Any,
            HirExpr::PreIncrement(name) | HirExpr::PostIncrement(name) |
            HirExpr::PreDecrement(name) | HirExpr::PostDecrement(name) => {
                if let Some((_, true_ty)) = self.emitter.id_registry.get(name) {
                    true_ty.clone()
                } else {
                    Type::Double
                }
            }
            _ => expr.get_type(),
        }
    }

    pub(crate) fn box_if_needed(&mut self, ty: &Type) {
        match ty {
            Type::Double => {
                self.code.push(Instruction::Invokestatic(self.emitter.double_value_of));
            }
            Type::Bool => self.code.push(Instruction::Invokestatic(self.emitter.boolean_value_of)),
            Type::Int => {
                self.code.push(Instruction::I2d);
                self.code.push(Instruction::Invokestatic(self.emitter.double_value_of));
            }
            _ => {}
        }
    }

    pub(crate) fn unbox_if_needed(&mut self, ty: &Type) {
        match ty {
            Type::Double => {
                self.code.push(Instruction::Checkcast(self.emitter.cp.add_class("java/lang/Double").unwrap()));
                self.code.push(Instruction::Invokevirtual(self.emitter.double_double_value));
            }
            Type::Bool => {
                self.code.push(Instruction::Checkcast(self.emitter.boolean_class));
                self.code.push(Instruction::Invokevirtual(self.emitter.boolean_boolean_value));
            }
            Type::StringTy => {
                let cls = self.emitter.cp.add_class("java/lang/String").unwrap();
                self.code.push(Instruction::Checkcast(cls));
            }
            Type::Array(_) => {
                let cls = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                self.code.push(Instruction::Checkcast(cls));
            }
            Type::Class(name) => {
                let cls = self.emitter.cp.add_class(name).unwrap();
                self.code.push(Instruction::Checkcast(cls));
            }
            _ => {}
        }
    }

    /// Convert a JS number (double) or boxed value to JVM int for array index operations
    pub(crate) fn unbox_to_int(&mut self, ty: &Type) {
        match ty {
            Type::Double => {
                self.code.push(Instruction::D2i);
            }
            Type::Int => {} // already int
            _ => {
                // Boxed value — cast to Double, then intValue()
                let double_class = self.emitter.cp.add_class("java/lang/Double").unwrap();
                let int_value = self.emitter.cp.add_method_ref(double_class, "intValue".to_string(), "()I".to_string()).unwrap();
                self.code.push(Instruction::Checkcast(double_class));
                self.code.push(Instruction::Invokevirtual(int_value));
            }
        }
    }
    
    #[allow(dead_code)]
    pub(crate) fn byte_offset(&self) -> usize {
        self.code.iter().map(inst_size).sum()
    }

    pub(crate) fn emit_expr_to_temp(&mut self, expr: &HirExpr) -> u8 {
        let temp_slot = self.emitter.local_slot;
        self.emitter.local_slot += 1;
        if let HirExpr::Spread(inner) = expr {
            self.emit_expr(inner);
            let ty = self.resolve_type(inner);
            self.box_if_needed(&ty);
        } else {
            self.emit_expr(expr);
            let ty = self.resolve_type(expr);
            self.box_if_needed(&ty);
        }
        self.code.push(Instruction::Astore(temp_slot));
        temp_slot
    }

    pub(crate) fn emit_cond_truthiness(&mut self, cond: &HirExpr) {
        self.emit_expr(cond);
        let cond_ty = self.resolve_type(cond);
        if cond_ty != Type::Bool && cond_ty != Type::Int {
            self.box_if_needed(&cond_ty);
            let ts_runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
            let is_truthy_method = self.emitter.cp.add_method_ref(
                ts_runtime_class,
                "isTruthy".to_string(),
                "(Ljava/lang/Object;)Z".to_string(),
            ).unwrap();
            self.code.push(Instruction::Invokestatic(is_truthy_method));
        }
    }

    pub(crate) fn coerce_to_expected_type(&mut self, current_ty: &Type, expected_ty: &Type) {
        if expected_ty == &Type::Any {
            self.box_if_needed(current_ty);
        } else if expected_ty != current_ty {
            if matches!(expected_ty, Type::Double | Type::Int | Type::Bool) && (current_ty == &Type::Any || !matches!(current_ty, Type::Double | Type::Int | Type::Bool)) {
                self.unbox_if_needed(expected_ty);
            }
        }
    }
}
