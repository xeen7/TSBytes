use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_if_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::If(test, cons, alt) => {
                            if let HirExpr::BoolLit(true) = test {
                                for s in cons { self.emit_stmt(s); }
                                return;
                            }
                            self.emit_cond_truthiness(test);
                            
                            let goto_idx = self.code.len();
                            self.code.push(Instruction::Goto(0)); // placeholder Ifeq
                            
                            for s in cons { self.emit_stmt(s); }
                            
                            if !alt.is_empty() {
                                let alt_goto = self.code.len();
                                self.code.push(Instruction::Goto(0));
                                
                                self.code[goto_idx] = Instruction::Ifeq(self.code.len() as u16);
                                
                                for s in alt { self.emit_stmt(s); }
                                
                                self.code[alt_goto] = Instruction::Goto(self.code.len() as u16);
                            } else {
                                self.code[goto_idx] = Instruction::Ifeq(self.code.len() as u16);
                            }
                            self.code.push(Instruction::Nop);
                        }
            _ => unreachable!(),
        }
    }
}
