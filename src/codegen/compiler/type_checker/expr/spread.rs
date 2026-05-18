use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_spread(&mut self, expr: &HirExpr) -> Type {
        self.check_expr(expr);
        Type::Any
    }
}
