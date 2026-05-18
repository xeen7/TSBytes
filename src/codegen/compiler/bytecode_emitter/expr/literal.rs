use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_literal(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::DoubleLit(d) => {
                            let idx = self.emitter.cp.add_double(*d).unwrap();
                            self.code.push(Instruction::Ldc2_w(idx));
                        }
            HirExpr::IntLit(i) => {
                            let idx = self.emitter.cp.add_integer(*i as i32).unwrap();
                            self.code.push(Instruction::Ldc_w(idx));
                        }
            HirExpr::StringLit(s) => {
                            let idx = self.emitter.cp.add_string(s).unwrap();
                            self.code.push(Instruction::Ldc_w(idx));
                        }
            HirExpr::BoolLit(b) => {
                            if *b { self.code.push(Instruction::Iconst_1); } else { self.code.push(Instruction::Iconst_0); }
                        }
            HirExpr::NullLit | HirExpr::UndefinedLit => {
                            self.code.push(Instruction::Aconst_null);
                        }
            _ => unreachable!(),
        }
    }
}
