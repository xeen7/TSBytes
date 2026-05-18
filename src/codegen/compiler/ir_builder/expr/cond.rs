use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_cond_expr(&mut self, cond: &CondExpr) -> HirExpr {
        let test = self.build_expr(&cond.test);
        let cons = self.build_expr(&cond.cons);
        let alt = self.build_expr(&cond.alt);
        let ty = cons.get_type();
        HirExpr::Ternary(Box::new(test), Box::new(cons), Box::new(alt), ty)
    }
}
