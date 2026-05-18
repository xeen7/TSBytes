use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_update_expr(&mut self, update: &UpdateExpr) -> HirExpr {
        if let Expr::Ident(ident) = &*update.arg {
            let name = ident.sym.to_string();
            match (update.prefix, update.op) {
                (true, UpdateOp::PlusPlus) => HirExpr::PreIncrement(name),
                (true, UpdateOp::MinusMinus) => HirExpr::PreDecrement(name),
                (false, UpdateOp::PlusPlus) => HirExpr::PostIncrement(name),
                (false, UpdateOp::MinusMinus) => HirExpr::PostDecrement(name),
            }
        } else {
            HirExpr::UndefinedLit
        }
    }
}
