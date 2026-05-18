use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_object_method(&mut self, _method: &str, args: &[HirExpr], ret_ty: &Type) -> Type {
        for arg in args { self.check_expr(arg); }
        ret_ty.clone()
    }
}
