use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_block(&mut self, block: &BlockStmt) -> Option<HirStmt> {
        self.env.enter_scope();
        let stmts = self.build_block_stmt(block);
        self.env.exit_scope();
        Some(HirStmt::If(HirExpr::BoolLit(true), stmts, vec![]))
    }
}
