use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_while(&mut self, test: &HirExpr, body: &[HirStmt]) {
        self.check_expr(test);
        self.env.enter_scope();
        for s in body { self.check_stmt(s); }
        self.env.exit_scope();
    }

    pub(crate) fn check_do_while(&mut self, body: &[HirStmt], test: &HirExpr) {
        self.env.enter_scope();
        for s in body { self.check_stmt(s); }
        self.env.exit_scope();
        self.check_expr(test);
    }
}
