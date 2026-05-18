use std::collections::HashSet;
use crate::codegen::compiler::mir::*;
use crate::codegen::compiler::ir::BinOp;

pub struct MirOptimizer;

impl MirOptimizer {
    pub fn optimize(mut functions: Vec<MirFunction>) -> Vec<MirFunction> {
        for func in &mut functions {
            let mut changed = true;
            while changed {
                changed = false;
                if Self::constant_folding(func) { changed = true; }
                if Self::constant_propagation(func) { changed = true; }
            }
            Self::dead_code_elimination(func);
        }
        functions
    }

    fn constant_propagation(func: &mut MirFunction) -> bool {
        let mut changed = false;
        for block in func.blocks.values_mut() {
            let mut const_vals: Vec<(Operand, Operand)> = Vec::new();
            for instr in &mut block.instrs {
                // Replace any operands with known constants
                match instr {
                    MirInstr::Assign(_, expr) => {
                        Self::propagate_in_expr(expr, &const_vals);
                    }
                    MirInstr::FieldSet(obj, _, val) => {
                        Self::propagate_in_op(obj, &const_vals);
                        Self::propagate_in_op(val, &const_vals);
                    }
                    MirInstr::IndexSet(obj, idx, val) => {
                        Self::propagate_in_op(obj, &const_vals);
                        Self::propagate_in_op(idx, &const_vals);
                        Self::propagate_in_op(val, &const_vals);
                    }
                    MirInstr::Call(_dest, _, args, _) | MirInstr::New(_dest, _, args, _) => {
                        for arg in args {
                            Self::propagate_in_op(arg, &const_vals);
                        }
                    }
                    MirInstr::MethodCall(_dest, obj, _, args, _) => {
                        Self::propagate_in_op(obj, &const_vals);
                        for arg in args {
                            Self::propagate_in_op(arg, &const_vals);
                        }
                    }
                    MirInstr::SuperCall(_, args, _) => {
                        for arg in args {
                            Self::propagate_in_op(arg, &const_vals);
                        }
                    }
                }
                
                // Track assignments to constants
                if let MirInstr::Assign(dest, MirExpr::Operand(Operand::Const(c, ty))) = instr {
                    const_vals.retain(|(k, _)| k != dest);
                    const_vals.push((dest.clone(), Operand::Const(c.clone(), ty.clone())));
                } else if let MirInstr::Assign(dest, _) = instr {
                    const_vals.retain(|(k, _)| k != dest); // invalidated
                }
            }
            
            // Also propagate to terminator
            match &mut block.terminator {
                Terminator::Branch(cond, _, _) => { if Self::propagate_in_op(cond, &const_vals) { changed = true; } }
                Terminator::Return(Some(op)) => { if Self::propagate_in_op(op, &const_vals) { changed = true; } }
                Terminator::Throw(op) => { if Self::propagate_in_op(op, &const_vals) { changed = true; } }
                _ => {}
            }
        }
        changed
    }

    fn propagate_in_expr(expr: &mut MirExpr, consts: &Vec<(Operand, Operand)>) -> bool {
        let mut changed = false;
        match expr {
            MirExpr::Operand(op) => { if Self::propagate_in_op(op, consts) { changed = true; } }
            MirExpr::BinOp(_, left, right, _) => {
                if Self::propagate_in_op(left, consts) { changed = true; }
                if Self::propagate_in_op(right, consts) { changed = true; }
            }
            MirExpr::UnaryOp(_, arg, _) => { if Self::propagate_in_op(arg, consts) { changed = true; } }
            MirExpr::FieldGet(obj, _, _) => { if Self::propagate_in_op(obj, consts) { changed = true; } }
            MirExpr::IndexGet(obj, idx, _) => {
                if Self::propagate_in_op(obj, consts) { changed = true; }
                if Self::propagate_in_op(idx, consts) { changed = true; }
            }
            MirExpr::ArrayLit(elems, _) => {
                for elem in elems { if Self::propagate_in_op(elem, consts) { changed = true; } }
            }
            MirExpr::ObjectLit(props, _) => {
                for (_, val) in props { if Self::propagate_in_op(val, consts) { changed = true; } }
            }
            MirExpr::Cast(op, _) => { if Self::propagate_in_op(op, consts) { changed = true; } }
            MirExpr::InstanceOf(op, _) => { if Self::propagate_in_op(op, consts) { changed = true; } }
        }
        changed
    }

    fn propagate_in_op(op: &mut Operand, consts: &Vec<(Operand, Operand)>) -> bool {
        if let Some((_, c)) = consts.iter().find(|(k, _)| k == op) {
            if op != c {
                *op = c.clone();
                return true;
            }
        }
        false
    }

    fn constant_folding(func: &mut MirFunction) -> bool {
        let mut changed = false;
        for block in func.blocks.values_mut() {
            for instr in &mut block.instrs {
                if let MirInstr::Assign(_, expr) = instr {
                    let mut replacement = None;
                    if let MirExpr::BinOp(op, Operand::Const(c1, _), Operand::Const(c2, _), res_ty) = expr {
                        match (c1, c2) {
                            (Constant::Double(d1), Constant::Double(d2)) => {
                                let folded = match op {
                                    BinOp::Add => Some(Constant::Double(*d1 + *d2)),
                                    BinOp::Sub => Some(Constant::Double(*d1 - *d2)),
                                    BinOp::Mul => Some(Constant::Double(*d1 * *d2)),
                                    BinOp::Div => Some(Constant::Double(*d1 / *d2)),
                                    _ => None,
                                };
                                if let Some(c) = folded {
                                    replacement = Some(MirExpr::Operand(Operand::Const(c, res_ty.clone())));
                                }
                            }
                            (Constant::Int(i1), Constant::Int(i2)) => {
                                let folded = match op {
                                    BinOp::Add => Some(Constant::Int(*i1 + *i2)),
                                    BinOp::Sub => Some(Constant::Int(*i1 - *i2)),
                                    BinOp::Mul => Some(Constant::Int(*i1 * *i2)),
                                    BinOp::Div => if *i2 != 0 { Some(Constant::Int(*i1 / *i2)) } else { None },
                                    _ => None,
                                };
                                if let Some(c) = folded {
                                    replacement = Some(MirExpr::Operand(Operand::Const(c, res_ty.clone())));
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(new_expr) = replacement {
                        *expr = new_expr;
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    fn dead_code_elimination(func: &mut MirFunction) {
        let mut reachable = HashSet::new();
        let mut worklist = vec![func.entry_block];

        // Reachability analysis
        while let Some(block_id) = worklist.pop() {
            if reachable.insert(block_id) {
                if let Some(block) = func.blocks.get(&block_id) {
                    match &block.terminator {
                        Terminator::Goto(tgt) => worklist.push(*tgt),
                        Terminator::Branch(Operand::Const(Constant::Bool(true), _), cons, _) => {
                            worklist.push(*cons);
                        }
                        Terminator::Branch(Operand::Const(Constant::Bool(false), _), _, alt) => {
                            worklist.push(*alt);
                        }
                        Terminator::Branch(_, cons, alt) => {
                            worklist.push(*cons);
                            worklist.push(*alt);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Remove unreachable blocks
        func.blocks.retain(|id, _| reachable.contains(id));
    }
}
