use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_update(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::PreIncrement(name) | HirExpr::PostIncrement(name) => {
                            if let Some(&(slot, ref var_ty)) = self.emitter.id_registry.get(name) {
                                if var_ty == &Type::Int {
                                    self.code.push(Instruction::Iload(slot));
                                    self.code.push(Instruction::Iconst_1);
                                    self.code.push(Instruction::Iadd);
                                    self.code.push(Instruction::Istore(slot));
                                    self.code.push(Instruction::Iload(slot));
                                } else {
                                    self.code.push(Instruction::Dload(slot));
                                    self.code.push(Instruction::Dconst_1);
                                    self.code.push(Instruction::Dadd);
                                    self.code.push(Instruction::Dstore(slot));
                                    self.code.push(Instruction::Dload(slot));
                                }
                            }
                        }
            HirExpr::PreDecrement(name) | HirExpr::PostDecrement(name) => {
                            if let Some(&(slot, ref var_ty)) = self.emitter.id_registry.get(name) {
                                if var_ty == &Type::Int {
                                    self.code.push(Instruction::Iload(slot));
                                    self.code.push(Instruction::Iconst_1);
                                    self.code.push(Instruction::Isub);
                                    self.code.push(Instruction::Istore(slot));
                                    self.code.push(Instruction::Iload(slot));
                                } else {
                                    self.code.push(Instruction::Dload(slot));
                                    self.code.push(Instruction::Dconst_1);
                                    self.code.push(Instruction::Dsub);
                                    self.code.push(Instruction::Dstore(slot));
                                    self.code.push(Instruction::Dload(slot));
                                }
                            }
                        }
            _ => unreachable!(),
        }
    }
}
