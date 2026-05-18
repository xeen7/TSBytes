use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_method_call(&mut self, obj: &HirExpr, _method: &str, args: &[HirExpr]) -> Type {
        self.check_expr(obj);
        for arg in args {
            self.check_expr(arg);
        }
        Type::Any
    }
}
