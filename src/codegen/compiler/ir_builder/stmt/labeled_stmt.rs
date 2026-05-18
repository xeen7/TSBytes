use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_labeled_stmt(&mut self, labeled_stmt: &LabeledStmt) -> Option<HirStmt> {
        let label = labeled_stmt.label.sym.to_string();
        let body = self.build_stmt(&labeled_stmt.body)?;
        Some(HirStmt::Labeled(label, Box::new(body)))
    }
}
