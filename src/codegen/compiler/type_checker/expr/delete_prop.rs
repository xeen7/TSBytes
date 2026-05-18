use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_delete_prop(&mut self, obj: &HirExpr, _prop: &str) -> Type {
        self.check_expr(obj);
        Type::Bool
    }
}
