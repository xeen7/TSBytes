use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_await_expr(&mut self, await_expr: &AwaitExpr) -> HirExpr {
        let inner = self.build_expr(&await_expr.arg);
        let ty = inner.get_type();
        HirExpr::Await(Box::new(inner), ty)
    }
}
