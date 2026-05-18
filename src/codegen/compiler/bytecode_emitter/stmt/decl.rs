use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_decl(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let(name, ty, expr) => {
                            self.emit_expr(expr);
                            let expr_ty = self.resolve_type(expr);
                            
                            if ty == &Type::Any && expr_ty.is_primitive() {
                                self.box_if_needed(&expr_ty);
                            } else if ty.is_primitive() && expr_ty == Type::Any {
                                self.unbox_if_needed(ty);
                            }
                            
                            let slot = self.emitter.local_slot;
                            if ty == &Type::Double {
                                self.emitter.local_slot += 2;
                            } else {
                                self.emitter.local_slot += 1;
                            }
                            self.emitter.id_registry.insert(name.clone(), (slot, ty.clone()));
                            
                            match ty {
                                Type::Double => self.code.push(Instruction::Dstore(slot)),
                                Type::Int => self.code.push(Instruction::Istore(slot)),
                                Type::Bool => self.code.push(Instruction::Istore(slot)),
                                _ => self.code.push(Instruction::Astore(slot)),
                            }
                        }
            HirStmt::Const(name, ty, expr) => {
                            // Const is identical to Let at bytecode level
                            self.emit_expr(expr);
                            let expr_ty = self.resolve_type(expr);
                            
                            if ty == &Type::Any && expr_ty.is_primitive() {
                                self.box_if_needed(&expr_ty);
                            } else if ty.is_primitive() && expr_ty == Type::Any {
                                self.unbox_if_needed(ty);
                            }
                            
                            let slot = self.emitter.local_slot;
                            if ty == &Type::Double {
                                self.emitter.local_slot += 2;
                            } else {
                                self.emitter.local_slot += 1;
                            }
                            self.emitter.id_registry.insert(name.clone(), (slot, ty.clone()));
                            match ty {
                                Type::Double => self.code.push(Instruction::Dstore(slot)),
                                Type::Int | Type::Bool => self.code.push(Instruction::Istore(slot)),
                                _ => self.code.push(Instruction::Astore(slot)),
                            }
                        }
            _ => unreachable!(),
        }
    }
}
