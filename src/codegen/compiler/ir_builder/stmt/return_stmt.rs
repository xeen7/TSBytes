use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_return_stmt(&mut self, ret_stmt: &ReturnStmt) -> Option<HirStmt> {
        let expr = ret_stmt.arg.as_ref().map(|e| self.build_expr(e));
        Some(HirStmt::Return(expr))
    }
}
