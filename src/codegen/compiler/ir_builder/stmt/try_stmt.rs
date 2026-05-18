use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_try_stmt(&mut self, try_stmt: &TryStmt) -> Option<HirStmt> {
        let try_block = self.build_block_stmt(&try_stmt.block);
        
        let mut catch_param = None;
        let mut catch_block = Vec::new();
        if let Some(handler) = &try_stmt.handler {
            if let Some(Pat::Ident(ident)) = &handler.param {
                let name = ident.id.sym.to_string();
                catch_param = Some(name.clone());
                self.env.enter_scope();
                self.env.bind(name, Type::Class("java/lang/Exception".to_string()));
            }
            catch_block = self.build_block_stmt(&handler.body);
            if catch_param.is_some() {
                self.env.exit_scope();
            }
        } else {
            catch_param = Some("__NO_CATCH__".to_string());
        }
        
        let finally_block = try_stmt.finalizer.as_ref().map(|b| self.build_block_stmt(b)).unwrap_or_default();
        Some(HirStmt::TryCatch(try_block, catch_param, catch_block, finally_block))
    }
}
