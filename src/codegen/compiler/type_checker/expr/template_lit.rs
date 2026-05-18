use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_template_lit(&mut self, parts: &[HirExpr]) -> Type {
        for part in parts { self.check_expr(part); }
        Type::StringTy
    }
}
