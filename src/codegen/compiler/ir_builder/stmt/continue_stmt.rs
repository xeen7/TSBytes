use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_continue_stmt(&mut self, continue_stmt: &ContinueStmt) -> Option<HirStmt> {
        let label = continue_stmt.label.as_ref().map(|l| l.sym.to_string());
        Some(HirStmt::Continue(label))
    }
}
