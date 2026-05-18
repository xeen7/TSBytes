use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_tpl_expr(&mut self, tpl: &Tpl) -> HirExpr {
        let mut parts = Vec::new();
        for i in 0..tpl.quasis.len() {
            let raw = tpl.quasis[i].raw.to_string();
            if !raw.is_empty() {
                parts.push(HirExpr::StringLit(raw));
            }
            if i < tpl.exprs.len() {
                parts.push(self.build_expr(&tpl.exprs[i]));
            }
        }
        HirExpr::TemplateLit(parts)
    }

    pub(crate) fn build_tagged_tpl_expr(&mut self, tagged_tpl: &TaggedTpl) -> HirExpr {
        let tag = self.build_expr(&tagged_tpl.tag);
        let mut quasis = Vec::new();
        for q in &tagged_tpl.tpl.quasis {
            quasis.push(HirExpr::StringLit(q.raw.to_string()));
        }
        let mut exprs = Vec::new();
        for e in &tagged_tpl.tpl.exprs {
            exprs.push(self.build_expr(e));
        }
        HirExpr::TaggedTemplate(Box::new(tag), quasis, exprs)
    }

    pub(crate) fn build_yield_expr(&mut self, yield_expr: &YieldExpr) -> HirExpr {
        let arg = yield_expr.arg.as_ref().map(|a| Box::new(self.build_expr(a)));
        HirExpr::Yield(arg, yield_expr.delegate)
    }
}
