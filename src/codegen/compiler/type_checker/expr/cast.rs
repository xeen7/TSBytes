use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_cast(&mut self, expr: &HirExpr, cast_ty: &Type) -> Type {
        self.check_expr(expr);
        cast_ty.clone()
    }
}
