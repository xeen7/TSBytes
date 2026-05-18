use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_instance_of(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::InstanceOf(expr, class_name) => {
                            self.emit_expr(expr);
                            let full_class_name = match class_name.as_str() {
                                "Date" => "com/tsdroid/runtime/TsDate".to_string(),
                                "Map" => "com/tsdroid/runtime/TsMap".to_string(),
                                "Set" => "com/tsdroid/runtime/TsSet".to_string(),
                                "WeakMap" => "com/tsdroid/runtime/TsWeakMap".to_string(),
                                "WeakSet" => "com/tsdroid/runtime/TsWeakSet".to_string(),
                                "RegExp" => "com/tsdroid/runtime/TsRegExp".to_string(),
                                "Error" => "com/tsdroid/runtime/TsError".to_string(),
                                _ => {
                                    if class_name.contains('/') {
                                        class_name.clone()
                                    } else {
                                        let package_name = self.emitter.class_path.as_ref().unwrap().rsplit_once('/').unwrap().0;
                                        format!("{}/{}", package_name, class_name)
                                    }
                                }
                            };
                            let class_idx = self.emitter.cp.add_class(&full_class_name).unwrap();
                            self.code.push(Instruction::Instanceof(class_idx));
                        }
            _ => unreachable!(),
        }
    }
}
