use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_break_stmt(&mut self, break_stmt: &BreakStmt) -> Option<HirStmt> {
        let label = break_stmt.label.as_ref().map(|l| l.sym.to_string());
        Some(HirStmt::Break(label))
    }
}
