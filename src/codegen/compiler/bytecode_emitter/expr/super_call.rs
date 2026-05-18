use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_super_call(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::SuperCall(method, args, _ty) => {
                let mut temp_slots = Vec::new();
                for arg in args {
                    temp_slots.push(self.emit_expr_to_temp(arg));
                }

                self.code.push(Instruction::Aload_0);

                self.code.push(Instruction::Bipush(args.len() as i8));
                let obj_class = self.emitter.cp.add_class("java/lang/Object").unwrap();
                self.code.push(Instruction::Anewarray(obj_class));
                for (i, &temp_slot) in temp_slots.iter().enumerate() {
                    self.code.push(Instruction::Dup);
                    self.code.push(Instruction::Bipush(i as i8));
                    self.code.push(Instruction::Aload(temp_slot));
                    self.code.push(Instruction::Aastore);
                }

                let super_class_name = self.emitter.super_class_path.as_ref().unwrap();
                let super_class = self.emitter.cp.add_class(super_class_name).unwrap();
                let is_ctor = method == "<init>";
                let desc = if is_ctor {
                    if super_class_name == "java/lang/Object" {
                        "()V".to_string()
                    } else {
                        "([Ljava/lang/Object;)V".to_string()
                    }
                } else {
                    "([Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                };
                let m = self.emitter.cp.add_method_ref(super_class, method.clone(), desc).unwrap();
                self.code.push(Instruction::Invokespecial(m));

                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
            }
            _ => unreachable!(),
        }
    }
}
