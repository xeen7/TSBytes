use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_decl(&mut self, name: &str, ty: &Type, expr: &HirExpr) {
        let expr_ty = self.check_expr(expr);
        if !expr_ty.is_assignable_to(ty) {
            self.errors.push(format!("Type error: Cannot assign type '{:?}' to variable '{}' of type '{:?}'", expr_ty, name, ty));
        }
        self.env.bind(name.to_string(), ty.clone());
    }
}
