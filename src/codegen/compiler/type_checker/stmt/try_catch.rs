use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_try_catch(&mut self, try_blk: &[HirStmt], catch_var: &Option<String>, catch_blk: &[HirStmt], finally_blk: &[HirStmt]) {
        self.env.enter_scope();
        for s in try_blk { self.check_stmt(s); }
        self.env.exit_scope();

        self.env.enter_scope();
        if let Some(var) = catch_var {
            self.env.bind(var.clone(), Type::Any);
        }
        for s in catch_blk { self.check_stmt(s); }
        self.env.exit_scope();

        self.env.enter_scope();
        for s in finally_blk { self.check_stmt(s); }
        self.env.exit_scope();
    }
}
