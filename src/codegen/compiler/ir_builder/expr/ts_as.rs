use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ts_as_expr(&mut self, ts_as: &TsAsExpr) -> HirExpr {
        let expr = self.build_expr(&ts_as.expr);
        let ty = self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: ts_as.type_ann.clone() }));
        HirExpr::Cast(Box::new(expr), ty)
    }
}
