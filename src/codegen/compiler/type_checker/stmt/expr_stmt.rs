use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_expr_stmt(&mut self, expr: &HirExpr) {
        self.check_expr(expr);
    }
}
