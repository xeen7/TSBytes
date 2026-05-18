use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_delete_prop(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::DeleteProp(obj, prop_name) => {
                            self.emit_expr(obj);
                            let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                            let delete_m = self.emitter.cp.add_method_ref(runtime_class, "deleteProperty".to_string(), "(Ljava/lang/Object;Ljava/lang/String;)Z".to_string()).unwrap();
                            let key_idx = self.emitter.cp.add_string(prop_name).unwrap();
                            self.code.push(Instruction::Ldc_w(key_idx));
                            self.code.push(Instruction::Invokestatic(delete_m));
                        }
            _ => unreachable!(),
        }
    }
}
