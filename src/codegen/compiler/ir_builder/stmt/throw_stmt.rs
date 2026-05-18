use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_throw_stmt(&mut self, throw_stmt: &ThrowStmt) -> Option<HirStmt> {
        Some(HirStmt::Throw(self.build_expr(&throw_stmt.arg)))
    }
}
