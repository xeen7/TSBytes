use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_array_lit_expr(&mut self, arr: &ArrayLit) -> HirExpr {
        let mut elems = Vec::new();
        let mut elem_type = Type::Any;
        for elem in &arr.elems {
            if let Some(e) = elem {
                let mut expr = self.build_expr(&e.expr);
                if e.spread.is_some() {
                    expr = HirExpr::Spread(Box::new(expr));
                }
                elem_type = expr.get_type();
                elems.push(expr);
            }
        }
        HirExpr::ArrayLit(elems, Type::Array(Box::new(elem_type)))
    }
}
