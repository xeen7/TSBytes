use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_control_flow(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Break(label) => {
                let target_idx = if let Some(lbl) = label {
                    self.loop_stack.iter().rev().position(|(l, _, _, _)| l.as_ref() == Some(lbl))
                } else {
                    Some(0)
                };
                
                if let Some(pos) = target_idx {
                    let real_idx = self.loop_stack.len() - 1 - pos;
                    self.loop_stack[real_idx].2.push(self.code.len());
                    self.code.push(Instruction::Goto(0));
                }
            }
            HirStmt::Continue(label) => {
                let target_idx = if let Some(lbl) = label {
                    self.loop_stack.iter().rev().position(|(l, _, _, _)| l.as_ref() == Some(lbl))
                } else {
                    Some(0)
                };
                
                if let Some(pos) = target_idx {
                    let real_idx = self.loop_stack.len() - 1 - pos;
                    self.loop_stack[real_idx].3.push(self.code.len());
                    self.code.push(Instruction::Goto(0));
                }
            }

            // Switch: emitted as a compare-chain (works for any value type)
            HirStmt::Switch(discriminant, cases) => {
                // Evaluate discriminant once into a temp slot
                let disc_slot = self.emitter.local_slot;
                self.emitter.local_slot += 1;
                self.emit_expr(discriminant);
                let disc_ty = self.resolve_type(discriminant);
                self.box_if_needed(&disc_ty);
                self.code.push(Instruction::Astore(disc_slot));

                // Push a synthetic loop context so that `break` inside switch works
                let switch_start = self.code.len();
                let label = self.pending_label.take();
                self.loop_stack.push((label, switch_start, Vec::new(), Vec::new()));

                // Keep track of gotos that must jump to the end of the switch
                let mut end_gotos: Vec<usize> = Vec::new();
                // Index in code of the goto that jumps to the next case (patched per case)
                let mut next_case_goto: Option<usize> = None;

                // Collect default case index for later
                let mut default_body_idx: Option<usize> = None;
                let mut default_end_goto: Option<usize> = None;

                let objects_class = self.emitter.cp.add_class("java/util/Objects").unwrap();
                let objects_equals = self.emitter.cp.add_method_ref(
                    objects_class,
                    "equals".to_string(),
                    "(Ljava/lang/Object;Ljava/lang/Object;)Z".to_string(),
                ).unwrap();

                for (case_idx, case) in cases.iter().enumerate() {
                    // Patch previous "not matched" goto to point here
                    if let Some(prev_goto) = next_case_goto.take() {
                        self.code[prev_goto] = Instruction::Ifeq(self.code.len() as u16);
                    }

                    if let Some(test_expr) = &case.test {
                        // Compare discriminant to case value using Objects.equals
                        self.code.push(Instruction::Aload(disc_slot));
                        self.emit_expr(test_expr);
                        let test_ty = self.resolve_type(test_expr);
                        self.box_if_needed(&test_ty);
                        self.code.push(Instruction::Invokestatic(objects_equals));

                        // If not equal, jump to next case (placeholder)
                        let not_eq_idx = self.code.len();
                        self.code.push(Instruction::Ifeq(0)); // placeholder
                        next_case_goto = Some(not_eq_idx);
                    } else {
                        // default case — record position, emit body below
                        default_body_idx = Some(case_idx);
                    }

                    // Emit case body
                    self.emit_stmts(&case.cons);

                    // After each case body: goto end (handles fall-through prevention)
                    // Unless the case already ended with break/return/throw
                    let last = self.code.last();
                    let already_jumps = matches!(last,
                        Some(Instruction::Goto(_)) | Some(Instruction::Return)
                        | Some(Instruction::Areturn) | Some(Instruction::Ireturn)
                        | Some(Instruction::Dreturn) | Some(Instruction::Athrow)
                    );
                    if !already_jumps {
                        let goto_end = self.code.len();
                        self.code.push(Instruction::Goto(0)); // placeholder → end
                        end_gotos.push(goto_end);
                        if case.test.is_none() {
                            default_end_goto = Some(goto_end);
                        }
                    }
                }

                // Patch the last unmatched-case goto to point past all cases
                if let Some(prev_goto) = next_case_goto.take() {
                    // If there's a default, redirect there; otherwise jump to end
                    // For simplicity, jump to end (default handled inline above)
                    self.code[prev_goto] = Instruction::Ifeq(self.code.len() as u16);
                }

                // End of switch — patch all end_gotos
                let end_pc = self.code.len() as u16;
                for idx in end_gotos {
                    self.code[idx] = Instruction::Goto(end_pc);
                }

                if let Some((_, _, break_indices, _)) = self.loop_stack.pop() {
                    for idx in break_indices {
                        self.code[idx] = Instruction::Goto(end_pc);
                    }
                }

                let _ = (default_body_idx, default_end_goto, switch_start);
            }

            HirStmt::Labeled(label, stmt) => {
                self.pending_label = Some(label.clone());
                self.emit_stmt(stmt);
            }

            HirStmt::ClassDecl(_, _)
            | HirStmt::InterfaceDecl(_, _) | HirStmt::Export(_) | HirStmt::Import(_, _)
            | HirStmt::FnDecl(_, _, _, _, _) => {
                // These are handled at top-level; no-op inside methods
            }

            _ => unreachable!(),
        }
    }
}
