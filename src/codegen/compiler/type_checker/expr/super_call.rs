use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_super_call(&mut self, _name: &str, args: &[HirExpr]) -> Type {
        for arg in args { self.check_expr(arg); }
        Type::Any
    }
}
