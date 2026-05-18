use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_for_in_stmt(&mut self, for_in: &ForInStmt) -> Option<HirStmt> {
        let mut var_name = None;

        match &for_in.left {
            ForHead::Pat(pat) => {
                if let Pat::Ident(ident) = &**pat {
                    var_name = Some(ident.id.sym.to_string());
                }
            }
            ForHead::VarDecl(var_decl) => {
                if let Some(decl) = var_decl.decls.first() {
                    if let Pat::Ident(ident) = &decl.name {
                        var_name = Some(ident.id.sym.to_string());
                    }
                }
            }
            _ => {}
        }

        if let Some(name) = var_name {
            let obj = self.build_expr(&for_in.right);
            let body = self.build_stmt(&for_in.body).map(|s| vec![s]).unwrap_or_default();
            return Some(HirStmt::ForIn(name, obj, body));
        }
        None
    }
}
