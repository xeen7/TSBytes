use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_export(&mut self, stmt: &HirStmt) {
        self.check_stmt(stmt);
    }

    pub(crate) fn check_import(&mut self, _bindings: &[ImportBinding], _source: &str) {
        // No checks needed for import currently
    }
}
