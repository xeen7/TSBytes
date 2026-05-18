use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_instance_of(&mut self, expr: &HirExpr, _class: &str) -> Type {
        self.check_expr(expr);
        Type::Bool
    }
}
