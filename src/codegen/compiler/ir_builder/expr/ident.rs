use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ident_expr(&mut self, ident: &Ident) -> HirExpr {
        let name = ident.sym.to_string();
        if name == "undefined" {
            HirExpr::UndefinedLit
        } else {
            let ty = self.env.get_type(&name);
            HirExpr::Var(name, ty)
        }
    }
}
