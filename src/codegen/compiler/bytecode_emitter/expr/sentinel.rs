use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_sentinel(&mut self, _expr: &HirExpr) {
        let ts_gen_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsGenerator").unwrap();
        let sentinel_field_ref = self.emitter.cp.add_field_ref(
            ts_gen_class,
            "SENTINEL".to_string(),
            "Ljava/lang/Object;".to_string(),
        ).unwrap();
        self.code.push(Instruction::Getstatic(sentinel_field_ref));
    }
}
