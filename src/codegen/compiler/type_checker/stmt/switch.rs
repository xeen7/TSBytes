use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_switch(&mut self, expr: &HirExpr, cases: &[SwitchCase]) {
        self.check_expr(expr);
        for case in cases {
            if let Some(c_expr) = &case.test {
                self.check_expr(c_expr);
            }
            self.env.enter_scope();
            for s in &case.cons { self.check_stmt(s); }
            self.env.exit_scope();
        }
    }
}
