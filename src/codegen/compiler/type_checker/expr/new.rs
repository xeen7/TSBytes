use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_new(&mut self, class: &str, args: &[HirExpr]) -> Type {
        for arg in args { self.check_expr(arg); }
        Type::Class(class.to_string())
    }
}
