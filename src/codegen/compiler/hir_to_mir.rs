use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::mir::*;

pub struct HirToMir {
    current_fn: MirFunction,
    current_block: BasicBlockId,
    next_temp: usize,
    next_block: usize,
    functions: Vec<MirFunction>,
    loop_stack: Vec<(BasicBlockId, BasicBlockId)>, // (continue_tgt, break_tgt)
}

impl HirToMir {
    pub fn new() -> Self {
        Self {
            current_fn: MirFunction::new("".to_string()),
            current_block: BasicBlockId(0),
            next_temp: 0,
            next_block: 1,
            functions: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn lower(mut self, stmts: &[HirStmt]) -> Vec<MirFunction> {
        let global_name = "$module_init".to_string();
        self.start_function(global_name);
        
        for stmt in stmts {
            self.lower_stmt(stmt);
        }
        
        self.end_block(Terminator::Return(None));
        self.functions.push(self.current_fn.clone());
        self.functions
    }

    fn new_temp(&mut self, ty: Type) -> Operand {
        let t = TempId(self.next_temp);
        self.next_temp += 1;
        Operand::Temp(t, ty)
    }

    fn new_block(&mut self) -> BasicBlockId {
        let b = BasicBlockId(self.next_block);
        self.next_block += 1;
        self.current_fn.blocks.insert(b, BasicBlock {
            id: b,
            instrs: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        b
    }

    fn emit(&mut self, instr: MirInstr) {
        if let Some(block) = self.current_fn.blocks.get_mut(&self.current_block) {
            block.instrs.push(instr);
        }
    }

    fn end_block(&mut self, term: Terminator) {
        if let Some(block) = self.current_fn.blocks.get_mut(&self.current_block) {
            if block.terminator == Terminator::Unreachable {
                block.terminator = term;
            }
        }
    }

    fn start_function(&mut self, name: String) {
        self.current_fn = MirFunction::new(name);
        self.next_temp = 0;
        self.next_block = 1;
        self.current_block = BasicBlockId(0);
        self.current_fn.blocks.insert(self.current_block, BasicBlock {
            id: self.current_block,
            instrs: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        self.loop_stack.clear();
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Expr(expr) => {
                self.lower_expr(expr);
            }
            HirStmt::Let(name, ty, expr) | HirStmt::Const(name, ty, expr) => {
                let op = self.lower_expr(expr);
                self.emit(MirInstr::Assign(Operand::Local(name.clone(), ty.clone()), MirExpr::Operand(op)));
            }
            HirStmt::Assign(name, expr) => {
                let op = self.lower_expr(expr);
                self.emit(MirInstr::Assign(Operand::Local(name.clone(), op.get_type()), MirExpr::Operand(op)));
            }
            HirStmt::CompoundAssign(name, op, expr) => {
                let right = self.lower_expr(expr);
                let left = Operand::Local(name.clone(), right.get_type());
                let bin_op = match op {
                    AssignOp::AddAssign => BinOp::Add,
                    AssignOp::SubAssign => BinOp::Sub,
                    AssignOp::MulAssign => BinOp::Mul,
                    AssignOp::DivAssign => BinOp::Div,
                    AssignOp::ModAssign => BinOp::Mod,
                    _ => BinOp::Add,
                };
                let temp = self.new_temp(right.get_type());
                self.emit(MirInstr::Assign(temp.clone(), MirExpr::BinOp(bin_op, left.clone(), right, temp.get_type())));
                self.emit(MirInstr::Assign(left, MirExpr::Operand(temp)));
            }
            HirStmt::FieldAssign(obj_expr, field, val_expr) => {
                let obj = self.lower_expr(obj_expr);
                let val = self.lower_expr(val_expr);
                self.emit(MirInstr::FieldSet(obj, field.clone(), val));
            }
            HirStmt::Return(expr_opt) => {
                let op = expr_opt.as_ref().map(|e| self.lower_expr(e));
                self.end_block(Terminator::Return(op));
                self.current_block = self.new_block(); // unreachable block after return
            }
            HirStmt::If(test, cons, alt) => {
                let cond = self.lower_expr(test);
                let then_block = self.new_block();
                let else_block = self.new_block();
                let merge_block = self.new_block();

                self.end_block(Terminator::Branch(cond, then_block, else_block));

                // Then
                self.current_block = then_block;
                for s in cons { self.lower_stmt(s); }
                self.end_block(Terminator::Goto(merge_block));

                // Else
                self.current_block = else_block;
                for s in alt { self.lower_stmt(s); }
                self.end_block(Terminator::Goto(merge_block));

                self.current_block = merge_block;
            }
            HirStmt::While(test, body) => {
                let cond_block = self.new_block();
                let body_block = self.new_block();
                let end_block = self.new_block();

                self.end_block(Terminator::Goto(cond_block));
                
                self.current_block = cond_block;
                let cond = self.lower_expr(test);
                self.end_block(Terminator::Branch(cond, body_block, end_block));

                self.current_block = body_block;
                self.loop_stack.push((cond_block, end_block));
                for s in body { self.lower_stmt(s); }
                self.loop_stack.pop();
                self.end_block(Terminator::Goto(cond_block));

                self.current_block = end_block;
            }
            HirStmt::Break(_) => {
                if let Some((_, end_block)) = self.loop_stack.last() {
                    self.end_block(Terminator::Goto(*end_block));
                }
                self.current_block = self.new_block();
            }
            HirStmt::Continue(_) => {
                if let Some((cond_block, _)) = self.loop_stack.last() {
                    self.end_block(Terminator::Goto(*cond_block));
                }
                self.current_block = self.new_block();
            }
            HirStmt::FnDecl(name, _, _, body, _) => {
                let prev_fn = self.current_fn.clone();
                let prev_block = self.current_block;
                let prev_next = self.next_block;
                let prev_temp = self.next_temp;

                self.start_function(name.clone());
                for s in body {
                    self.lower_stmt(s);
                }
                self.end_block(Terminator::Return(None));
                self.functions.push(self.current_fn.clone());

                self.current_fn = prev_fn;
                self.current_block = prev_block;
                self.next_block = prev_next;
                self.next_temp = prev_temp;
            }
            _ => { /* Ignore complex stmts for now */ }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Operand {
        match expr {
            HirExpr::IntLit(n) => Operand::Const(Constant::Int(*n), Type::Int),
            HirExpr::DoubleLit(n) => Operand::Const(Constant::Double(*n), Type::Double),
            HirExpr::StringLit(s) => Operand::Const(Constant::String(s.clone()), Type::StringTy),
            HirExpr::BoolLit(b) => Operand::Const(Constant::Bool(*b), Type::Bool),
            HirExpr::NullLit => Operand::Const(Constant::Null, Type::Null),
            HirExpr::UndefinedLit => Operand::Const(Constant::Undefined, Type::Undefined),
            HirExpr::Var(name, ty) => Operand::Local(name.clone(), ty.clone()),
            HirExpr::This(ty) => Operand::This(ty.clone()),
            HirExpr::BinOp(op, left, right, ty) => {
                let l = self.lower_expr(left);
                let r = self.lower_expr(right);
                let temp = self.new_temp(ty.clone());
                self.emit(MirInstr::Assign(temp.clone(), MirExpr::BinOp(op.clone(), l, r, ty.clone())));
                temp
            }
            HirExpr::UnaryOp(op, arg, ty) => {
                let a = self.lower_expr(arg);
                let temp = self.new_temp(ty.clone());
                self.emit(MirInstr::Assign(temp.clone(), MirExpr::UnaryOp(op.clone(), a, ty.clone())));
                temp
            }
            HirExpr::FieldGet(obj, prop, ty) => {
                let o = self.lower_expr(obj);
                let temp = self.new_temp(ty.clone());
                self.emit(MirInstr::Assign(temp.clone(), MirExpr::FieldGet(o, prop.clone(), ty.clone())));
                temp
            }
            HirExpr::Call(name, args, ty) => {
                let mut ops = Vec::new();
                for arg in args {
                    ops.push(self.lower_expr(arg));
                }
                let temp = self.new_temp(ty.clone());
                self.emit(MirInstr::Call(Some(temp.clone()), name.clone(), ops, ty.clone()));
                temp
            }
            HirExpr::MethodCall(obj, method, args, ty) => {
                let o = self.lower_expr(obj);
                let mut ops = Vec::new();
                for arg in args {
                    ops.push(self.lower_expr(arg));
                }
                let temp = self.new_temp(ty.clone());
                self.emit(MirInstr::MethodCall(Some(temp.clone()), o, method.clone(), ops, ty.clone()));
                temp
            }
            _ => {
                // Fallback for unhandled expressions
                Operand::Const(Constant::Undefined, Type::Undefined)
            }
        }
    }
}
