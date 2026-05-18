use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_assign(&mut self, name: &str, expr: &HirExpr) {
        let expr_ty = self.check_expr(expr);
        let target_ty = self.env.get_type(name);
        if !expr_ty.is_assignable_to(&target_ty) {
            self.errors.push(format!("Type error: Cannot assign type '{:?}' to variable '{}' of type '{:?}'", expr_ty, name, target_ty));
        }
    }

    pub(crate) fn check_compound_assign(&mut self, name: &str, _op: &AssignOp, expr: &HirExpr) {
        let expr_ty = self.check_expr(expr);
        let target_ty = self.env.get_type(name);
        if !expr_ty.is_assignable_to(&target_ty) {
            self.errors.push(format!("Type error: Compound assignment type mismatch: '{:?}' to '{:?}'", expr_ty, target_ty));
        }
    }

    pub(crate) fn check_field_assign(&mut self, obj: &HirExpr, _field: &str, expr: &HirExpr) {
        self.check_expr(obj);
        self.check_expr(expr);
    }
}
