use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_unary_op(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::UnaryOp(op, arg, _ty) => {
                            self.emit_expr(arg);
                            match op {
                                UnaryOp::Not => {
                                    let arg_ty = self.resolve_type(arg);
                                    if arg_ty != Type::Bool && arg_ty != Type::Int {
                                        self.box_if_needed(&arg_ty);
                                        let ts_runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                        let is_truthy_method = self.emitter.cp.add_method_ref(
                                            ts_runtime_class,
                                            "isTruthy".to_string(),
                                            "(Ljava/lang/Object;)Z".to_string(),
                                        ).unwrap();
                                        self.code.push(Instruction::Invokestatic(is_truthy_method));
                                    }

                                    let ifeq_idx = self.code.len();
                                    self.code.push(Instruction::Ifeq(0)); // placeholder
                                    
                                    self.code.push(Instruction::Iconst_0);
                                    
                                    let goto_idx = self.code.len();
                                    self.code.push(Instruction::Goto(0)); // placeholder
                                    
                                    let false_target = self.code.len();
                                    self.code[ifeq_idx] = Instruction::Ifeq(false_target as u16);
                                    
                                    self.code.push(Instruction::Iconst_1);
                                    
                                    let end_target = self.code.len();
                                    self.code[goto_idx] = Instruction::Goto(end_target as u16);
                                }
                                UnaryOp::Neg => match self.resolve_type(arg) {
                                    Type::Double => self.code.push(Instruction::Dneg),
                                    Type::Int => self.code.push(Instruction::Ineg),
                                    _ => {}
                                },
                                UnaryOp::TypeOf => {
                                    let arg_ty = self.resolve_type(arg);
                                    self.box_if_needed(&arg_ty);
                                    
                                    let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                                    let typeof_method = self.emitter.cp.add_method_ref(
                                        ts_object_class,
                                        "typeofValue".to_string(),
                                        "(Ljava/lang/Object;)Ljava/lang/String;".to_string(),
                                    ).unwrap();
                                    self.code.push(Instruction::Invokestatic(typeof_method));
                                }
                                _ => {}
                            }
                        }
            _ => unreachable!(),
        }
    }
}
