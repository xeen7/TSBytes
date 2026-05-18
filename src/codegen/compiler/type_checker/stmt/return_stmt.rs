use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_return(&mut self, expr_opt: &Option<HirExpr>) {
        let ret_ty = if let Some(expr) = expr_opt {
            self.check_expr(expr)
        } else {
            Type::Void
        };
        if let Some(expected_ret) = &self.current_fn_return {
            if !ret_ty.is_assignable_to(expected_ret) {
                self.errors.push(format!("Type error: Cannot return type '{:?}' from function expecting '{:?}'", ret_ty, expected_ret));
            }
        }
    }
}
