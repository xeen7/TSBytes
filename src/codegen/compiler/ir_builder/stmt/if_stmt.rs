use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_if_stmt(&mut self, if_stmt: &IfStmt) -> Option<HirStmt> {
        let test = self.build_expr(&if_stmt.test);
        let cons = self.build_stmt(&if_stmt.cons).map(|s| vec![s]).unwrap_or_default();
        let alt = if_stmt.alt.as_ref().and_then(|a| self.build_stmt(a)).map(|s| vec![s]).unwrap_or_default();
        Some(HirStmt::If(test, cons, alt))
    }
}
