use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_while_stmt(&mut self, while_stmt: &WhileStmt) -> Option<HirStmt> {
        let test = self.build_expr(&while_stmt.test);
        let body = self.build_stmt(&while_stmt.body).map(|s| vec![s]).unwrap_or_default();
        Some(HirStmt::While(test, body))
    }
}
