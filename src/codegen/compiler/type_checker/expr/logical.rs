use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_logical(&mut self, l: &HirExpr, r: &HirExpr) -> Type {
        let lty = self.check_expr(l);
        let rty = self.check_expr(r);
        if lty == rty { lty } else { Type::Any }
    }
}
