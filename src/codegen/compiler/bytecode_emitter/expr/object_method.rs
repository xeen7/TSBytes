use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    fn emit_tsobject_expr(&mut self, expr: &HirExpr) {
        self.emit_expr(expr);
        let ty = self.resolve_type(expr);
        if !matches!(ty, Type::Class(..)) {
            let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
            self.code.push(Instruction::Checkcast(ts_object_class));
        }
    }

    pub(crate) fn emit_object_method(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::ObjectMethod(method_name, args, _ty) => {
                            let ts_object_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
                            let ts_obj_desc = format!("L{};", "com/tsdroid/runtime/TsObject");
                            match method_name.as_str() {
                                "keys" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "keys".to_string(), format!("({})Ljava/lang/Object;", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "values" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "values".to_string(), format!("({})Ljava/lang/Object;", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "entries" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "entries".to_string(), format!("({})Ljava/lang/Object;", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "assign" => {
                                    // Object.assign(target, source)
                                    self.emit_tsobject_expr(&args[0]);
                                    self.emit_tsobject_expr(&args[1]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "assign".to_string(), format!("({0}{0}){0}", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "create" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "create".to_string(), format!("({0}){0}", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "freeze" | "seal" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "freeze".to_string(), format!("({0}){0}", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "defineProperty" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    self.emit_expr(&args[1]);
                                    self.emit_expr(&args[2]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "defineProperty".to_string(), format!("({0}Ljava/lang/String;Ljava/lang/Object;){0}", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "getPrototypeOf" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "getPrototypeOf".to_string(), format!("({0}){0}", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "setPrototypeOf" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    self.emit_tsobject_expr(&args[1]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "setPrototypeOf".to_string(), format!("({0}{0})V", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "hasOwn" => {
                                    self.emit_tsobject_expr(&args[0]);
                                    self.emit_expr(&args[1]);
                                    let m = self.emitter.cp.add_method_ref(ts_object_class, "hasOwn".to_string(), format!("({}Ljava/lang/String;)Z", ts_obj_desc)).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                _ => {
                                    self.code.push(Instruction::Aconst_null); // unknown method
                                }
                            }
                        }
            _ => unreachable!(),
        }
    }
}
