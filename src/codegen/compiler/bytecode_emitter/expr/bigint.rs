use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_bigint(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::BigIntLit(s) => {
                let class_idx = self.emitter.cp.add_class("java/math/BigInteger").unwrap();
                self.code.push(Instruction::New(class_idx));
                self.code.push(Instruction::Dup);

                // Strip any trailing 'n' (e.g. 100n -> 100)
                let bigint_val = s.trim_end_matches('n');
                let str_idx = self.emitter.cp.add_string(bigint_val).unwrap();
                self.code.push(Instruction::Ldc_w(str_idx));

                let init_idx = self.emitter.cp.add_method_ref(class_idx, "<init>", "(Ljava/lang/String;)V").unwrap();
                self.code.push(Instruction::Invokespecial(init_idx));
            }
            _ => unreachable!(),
        }
    }
}
