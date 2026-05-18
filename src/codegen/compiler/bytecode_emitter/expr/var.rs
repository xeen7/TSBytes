use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_var(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Var(name, _ty) => {
                            if let Some((slot, true_ty)) = self.emitter.id_registry.get(name).cloned() {
                                match true_ty {
                                    Type::Double => self.code.push(Instruction::Dload(slot)),
                                    Type::Int | Type::Bool => self.code.push(Instruction::Iload(slot)),
                                    _ => self.code.push(Instruction::Aload(slot)),
                                }
                            } else {
                                self.code.push(Instruction::Aconst_null);
                            }
                        }
            _ => unreachable!(),
        }
    }
}
