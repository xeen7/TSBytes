use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_if(&mut self, test: &HirExpr, cons: &[HirStmt], alt: &[HirStmt]) {
        self.check_expr(test);
        
        self.env.enter_scope();
        for s in cons { self.check_stmt(s); }
        self.env.exit_scope();

        self.env.enter_scope();
        for s in alt { self.check_stmt(s); }
        self.env.exit_scope();
    }
}
