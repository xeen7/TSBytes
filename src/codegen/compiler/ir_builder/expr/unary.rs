use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_unary_expr(&mut self, unary: &UnaryExpr) -> HirExpr {
        if unary.op == swc_core::ecma::ast::UnaryOp::Delete {
            if let Expr::Member(member) = &*unary.arg {
                let obj = self.build_expr(&member.obj);
                if let MemberProp::Ident(ident) = &member.prop {
                    return HirExpr::DeleteProp(Box::new(obj), ident.sym.to_string());
                }
            }
        }
        let arg = self.build_expr(&unary.arg);
        let op = match unary.op {
            swc_core::ecma::ast::UnaryOp::Bang => crate::codegen::compiler::ir::UnaryOp::Not,
            swc_core::ecma::ast::UnaryOp::Minus => crate::codegen::compiler::ir::UnaryOp::Neg,
            swc_core::ecma::ast::UnaryOp::Plus => crate::codegen::compiler::ir::UnaryOp::Pos,
            swc_core::ecma::ast::UnaryOp::TypeOf => crate::codegen::compiler::ir::UnaryOp::TypeOf,
            swc_core::ecma::ast::UnaryOp::Void => crate::codegen::compiler::ir::UnaryOp::VoidOp,
            swc_core::ecma::ast::UnaryOp::Delete => crate::codegen::compiler::ir::UnaryOp::Delete,
            _ => crate::codegen::compiler::ir::UnaryOp::VoidOp,
        };
        let ty = match op {
            crate::codegen::compiler::ir::UnaryOp::Not => Type::Bool,
            crate::codegen::compiler::ir::UnaryOp::Neg | crate::codegen::compiler::ir::UnaryOp::Pos => Type::Double,
            crate::codegen::compiler::ir::UnaryOp::TypeOf => Type::StringTy,
            _ => Type::Any,
        };
        HirExpr::UnaryOp(op, Box::new(arg), ty)
    }
}
