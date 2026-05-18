use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_ternary(&mut self, cond: &HirExpr, cons: &HirExpr, alt: &HirExpr) -> Type {
        self.check_expr(cond);
        let c_ty = self.check_expr(cons);
        let a_ty = self.check_expr(alt);
        if c_ty == a_ty { c_ty } else { Type::Any }
    }
}
