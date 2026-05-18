use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_template_lit(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::TemplateLit(parts) => {
                            // StringBuilder pattern for string concatenation
                            let sb_class = self.emitter.cp.add_class("java/lang/StringBuilder").unwrap();
                            let sb_init = self.emitter.cp.add_method_ref(sb_class, "<init>".to_string(), "()V".to_string()).unwrap();
                            let sb_append = self.emitter.cp.add_method_ref(sb_class, "append".to_string(), "(Ljava/lang/String;)Ljava/lang/StringBuilder;".to_string()).unwrap();
                            let sb_tostring = self.emitter.cp.add_method_ref(sb_class, "toString".to_string(), "()Ljava/lang/String;".to_string()).unwrap();
                            let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                            let ts_to_string = self.emitter.cp.add_method_ref(runtime_class, "tsToString".to_string(), "(Ljava/lang/Object;)Ljava/lang/String;".to_string()).unwrap();
                            self.code.push(Instruction::New(sb_class));
                            self.code.push(Instruction::Dup);
                            self.code.push(Instruction::Invokespecial(sb_init));
                            for part in parts {
                                self.emit_expr(part);
                                let resolved_ty = self.resolve_type(part);
                                self.box_if_needed(&resolved_ty);
                                self.code.push(Instruction::Invokestatic(ts_to_string));
                                self.code.push(Instruction::Invokevirtual(sb_append));
                            }
                            self.code.push(Instruction::Invokevirtual(sb_tostring));
                        }
            _ => unreachable!(),
        }
    }
}
