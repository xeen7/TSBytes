use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_new_expr(&mut self, new_expr: &NewExpr) -> HirExpr {
        if let Expr::Ident(ident) = &*new_expr.callee {
            let class_name = ident.sym.to_string();
            let mut args = Vec::new();
            if let Some(call_args) = &new_expr.args {
                for arg in call_args {
                    args.push(self.build_expr(&arg.expr));
                }
            }
            HirExpr::New(class_name.clone(), args, Type::Class(class_name))
        } else {
            HirExpr::UndefinedLit
        }
    }
}
