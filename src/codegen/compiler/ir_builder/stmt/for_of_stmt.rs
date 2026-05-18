use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_for_of_stmt(&mut self, for_of: &ForOfStmt) -> Option<HirStmt> {
        let mut var_name = None;
        let mut ty = Type::Any;

        match &for_of.left {
            ForHead::Pat(pat) => {
                if let Pat::Ident(ident) = &**pat {
                    var_name = Some(ident.id.sym.to_string());
                    ty = self.extract_type(ident.type_ann.as_deref());
                }
            }
            ForHead::VarDecl(var_decl) => {
                if let Some(decl) = var_decl.decls.first() {
                    if let Pat::Ident(ident) = &decl.name {
                        var_name = Some(ident.id.sym.to_string());
                        ty = self.extract_type(ident.type_ann.as_deref());
                    }
                }
            }
            _ => {}
        }

        if let Some(name) = var_name {
            let iterable = self.build_expr(&for_of.right);
            let body = self.build_stmt(&for_of.body).map(|s| vec![s]).unwrap_or_default();
            if for_of.is_await {
                return Some(HirStmt::ForAwaitOf(name, ty, iterable, body));
            } else {
                return Some(HirStmt::ForOf(name, ty, iterable, body));
            }
        }
        None
    }
}
