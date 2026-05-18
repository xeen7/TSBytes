use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_switch_stmt(&mut self, switch_stmt: &SwitchStmt) -> Option<HirStmt> {
        let test = self.build_expr(&switch_stmt.discriminant);
        let mut cases = Vec::new();
        for case in &switch_stmt.cases {
            let case_test = case.test.as_ref().map(|t| self.build_expr(t));
            let mut case_cons = Vec::new();
            for stmt in &case.cons {
                if let Some(hir_stmt) = self.build_stmt(stmt) {
                    case_cons.push(hir_stmt);
                }
            }
            cases.push(crate::codegen::compiler::ir::SwitchCase { test: case_test, cons: case_cons });
        }
        Some(HirStmt::Switch(test, cases))
    }
}
