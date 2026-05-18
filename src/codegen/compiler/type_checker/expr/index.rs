use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_index_get(&mut self, obj: &HirExpr, idx: &HirExpr) -> Type {
        let obj_ty = self.check_expr(obj);
        self.check_expr(idx);
        if let Type::Array(inner) = obj_ty {
            (*inner).clone()
        } else {
            Type::Any
        }
    }

    pub(crate) fn check_index_set(&mut self, obj: &HirExpr, idx: &HirExpr, val: &HirExpr) -> Type {
        self.check_expr(obj);
        self.check_expr(idx);
        self.check_expr(val);
        Type::Any
    }
}
