use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_array_len(&mut self, obj: &HirExpr) -> Type {
        self.check_expr(obj);
        Type::Double
    }

    pub(crate) fn check_array_method(&mut self, obj: &HirExpr, _method: &str, args: &[HirExpr], ty: &Type) -> Type {
        self.check_expr(obj);
        for arg in args { self.check_expr(arg); }
        ty.clone()
    }
}
