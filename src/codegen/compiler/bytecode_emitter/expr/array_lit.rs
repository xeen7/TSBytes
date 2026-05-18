use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_array_lit(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::ArrayLit(elems, _ty) => {
                // Pre-evaluate elements on an empty stack
                let mut temp_slots = Vec::new();
                for elem in elems {
                    temp_slots.push(self.emit_expr_to_temp(elem));
                }

                self.code.push(Instruction::New(self.emitter.array_list_class));
                self.code.push(Instruction::Dup);
                self.code.push(Instruction::Invokespecial(self.emitter.array_list_init));

                for (elem, &temp_slot) in elems.iter().zip(temp_slots.iter()) {
                    if let HirExpr::Spread(_) = elem {
                        let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                        let spread_arr = self.emitter.cp.add_method_ref(
                            runtime_class, 
                            "spreadArray".to_string(), 
                            "(Ljava/util/ArrayList;Ljava/lang/Object;)Ljava/lang/Object;".to_string()
                        ).unwrap();
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Invokestatic(spread_arr));
                        self.code.push(Instruction::Pop);
                    } else {
                        self.code.push(Instruction::Dup);
                        self.code.push(Instruction::Aload(temp_slot));
                        self.code.push(Instruction::Invokevirtual(self.emitter.array_list_add));
                        self.code.push(Instruction::Pop);
                    }
                }

                // Free temp slots
                for _ in &temp_slots {
                    self.emitter.local_slot -= 1;
                }
            }
            _ => unreachable!(),
        }
    }
}
