use ristretto_classfile::attributes::{Instruction, CodeException};
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_try_catch(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::TryCatch(try_block, catch_param, catch_block, finally_block) => {
                let has_catch = catch_param.as_deref() != Some("__NO_CATCH__");
                
                let try_start = self.code.len();
                self.emit_stmts(try_block);
                let try_end = self.code.len();
                
                let try_terminated = if let Some(last_inst) = self.code.last() {
                    matches!(last_inst, 
                        Instruction::Athrow | Instruction::Return | Instruction::Ireturn |
                        Instruction::Areturn | Instruction::Dreturn | Instruction::Freturn |
                        Instruction::Lreturn
                    )
                } else {
                    false
                };

                let goto_normal_finally_idx = if has_catch && !try_terminated {
                    let idx = self.code.len();
                    self.code.push(Instruction::Goto(0));
                    Some(idx)
                } else {
                    None
                };
                
                if has_catch {
                    let handler_pc = self.code.len();
                    
                    if let Some(param) = catch_param {
                        let slot = self.emitter.local_slot;
                        self.emitter.local_slot += 1;
                        self.emitter.id_registry.insert(param.clone(), (slot, Type::Any));
                        
                        self.code.push(Instruction::Astore(slot));
                    } else {
                        self.code.push(Instruction::Pop);
                    }
                    
                    self.emit_stmts(catch_block);
                    
                    self.exceptions.push(CodeException {
                        start_pc: try_start as u16,
                        end_pc: try_end as u16,
                        handler_pc: handler_pc as u16,
                        catch_type: 0,
                    });
                }
                
                if let Some(idx) = goto_normal_finally_idx {
                    self.code[idx] = Instruction::Goto(self.code.len() as u16);
                }
                
                let normal_finally_start = self.code.len();
                if !finally_block.is_empty() {
                    self.emit_stmts(finally_block);
                }
                
                let goto_end_idx = if !finally_block.is_empty() {
                    let idx = self.code.len();
                    self.code.push(Instruction::Goto(0));
                    Some(idx)
                } else {
                    None
                };
                
                if !finally_block.is_empty() {
                    let finally_handler_start = self.code.len();
                    
                    let temp_exc_slot = self.emitter.local_slot;
                    self.emitter.local_slot += 1;
                    
                    self.code.push(Instruction::Astore(temp_exc_slot));
                    self.emit_stmts(finally_block);
                    self.code.push(Instruction::Aload(temp_exc_slot));
                    self.code.push(Instruction::Athrow);
                    
                    self.emitter.local_slot -= 1;
                    
                    // Register exception handler for finally covering try and catch blocks
                    self.exceptions.push(CodeException {
                        start_pc: try_start as u16,
                        end_pc: normal_finally_start as u16,
                        handler_pc: finally_handler_start as u16,
                        catch_type: 0,
                    });
                }
                
                if let Some(idx) = goto_end_idx {
                    self.code[idx] = Instruction::Goto(self.code.len() as u16);
                }
                
                self.code.push(Instruction::Nop);
            }
            _ => unreachable!(),
        }
    }
}
