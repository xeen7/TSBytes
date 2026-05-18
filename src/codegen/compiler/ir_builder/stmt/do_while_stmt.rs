use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_do_while_stmt(&mut self, do_while: &DoWhileStmt) -> Option<HirStmt> {
        let body = self.build_stmt(&do_while.body).map(|s| vec![s]).unwrap_or_default();
        let test = self.build_expr(&do_while.test);
        Some(HirStmt::DoWhile(body, test))
    }
}
