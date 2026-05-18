use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_optional_chain(&mut self, obj: &HirExpr, _prop: &str) -> Type {
        self.check_expr(obj);
        Type::Any
    }

    pub(crate) fn check_optional_call(&mut self, callee: &HirExpr, args: &[HirExpr]) -> Type {
        self.check_expr(callee);
        for arg in args {
            self.check_expr(arg);
        }
        Type::Any
    }

    pub(crate) fn check_dynamic_call(&mut self, callee: &HirExpr, args: &[HirExpr]) -> Type {
        self.check_expr(callee);
        for arg in args {
            self.check_expr(arg);
        }
        Type::Any
    }
}
