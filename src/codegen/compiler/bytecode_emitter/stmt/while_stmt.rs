use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_while_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::While(test, body) => {
                            let start_pc = self.code.len();
                            
                            self.emit_cond_truthiness(test);
                            
                            let goto_idx = self.code.len();
                            self.code.push(Instruction::Goto(0)); // Ifeq placeholder
                            
                            let label = self.pending_label.take();
                            self.loop_stack.push((label, start_pc, Vec::new(), Vec::new()));
                            
                            for s in body { self.emit_stmt(s); }
                            
                            let (_, _, break_indices, continue_indices) = self.loop_stack.pop().unwrap();
                            
                            self.code.push(Instruction::Goto(start_pc as u16)); // Backward jump
                            
                            let end_pc = self.code.len();
                            self.code[goto_idx] = Instruction::Ifeq(end_pc as u16); // jump past the Goto
                            
                            // Patch breaks
                            for idx in break_indices {
                                self.code[idx] = Instruction::Goto(end_pc as u16);
                            }
                            // Patch continues
                            for idx in continue_indices {
                                self.code[idx] = Instruction::Goto(start_pc as u16);
                            }
                        }
            HirStmt::DoWhile(body, test) => {
                            let start_pc = self.code.len();
                            let label = self.pending_label.take();
                            self.loop_stack.push((label, start_pc, Vec::new(), Vec::new()));
                            
                            for s in body { self.emit_stmt(s); }
                            
                            let continue_pc = self.code.len();
                            self.emit_cond_truthiness(test);
                            self.code.push(Instruction::Ifne(start_pc as u16));
                            
                            let end_pc = self.code.len();
                            let (_, _, break_indices, continue_indices) = self.loop_stack.pop().unwrap();
                            
                            for idx in break_indices {
                                self.code[idx] = Instruction::Goto(end_pc as u16);
                            }
                            for idx in continue_indices {
                                self.code[idx] = Instruction::Goto(continue_pc as u16);
                            }
                        }
            _ => unreachable!(),
        }
    }
}
