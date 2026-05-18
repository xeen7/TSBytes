use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_control_flow(&mut self, _stmt: &HirStmt) {
        // Nothing to check for break/continue
    }

    pub(crate) fn check_labeled(&mut self, _label: &str, s: &HirStmt) {
        self.check_stmt(s);
    }
}
