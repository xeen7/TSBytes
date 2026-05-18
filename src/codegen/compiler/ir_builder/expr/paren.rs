use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_paren_expr(&mut self, paren: &ParenExpr) -> HirExpr {
        self.build_expr(&paren.expr)
    }
}
