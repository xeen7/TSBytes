use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_bin_expr(&mut self, bin: &BinExpr) -> HirExpr {
        let left = self.build_expr(&bin.left);
        let right = self.build_expr(&bin.right);
        match bin.op {
            BinaryOp::LogicalAnd => HirExpr::LogicalAnd(Box::new(left), Box::new(right), Type::Any),
            BinaryOp::LogicalOr  => HirExpr::LogicalOr(Box::new(left), Box::new(right), Type::Any),
            BinaryOp::InstanceOf => {
                let class_name = match &right {
                    HirExpr::Var(name, _) => name.clone(),
                    HirExpr::StringLit(s)  => s.clone(),
                    _ => "java/lang/Object".to_string(),
                };
                HirExpr::InstanceOf(Box::new(left), class_name)
            }
            _ => {
                let op = match bin.op {
                    BinaryOp::Add               => BinOp::Add,
                    BinaryOp::Sub               => BinOp::Sub,
                    BinaryOp::Mul               => BinOp::Mul,
                    BinaryOp::Div               => BinOp::Div,
                    BinaryOp::Mod               => BinOp::Mod,
                    BinaryOp::Exp               => BinOp::Exp,
                    BinaryOp::EqEq
                    | BinaryOp::EqEqEq          => BinOp::Eq,
                    BinaryOp::NotEq
                    | BinaryOp::NotEqEq         => BinOp::Ne,
                    BinaryOp::Lt                => BinOp::Lt,
                    BinaryOp::LtEq              => BinOp::Le,
                    BinaryOp::Gt                => BinOp::Gt,
                    BinaryOp::GtEq              => BinOp::Ge,
                    BinaryOp::In                => BinOp::In,
                    BinaryOp::NullishCoalescing => BinOp::NullishCoalesce,
                    // Bitwise
                    BinaryOp::BitAnd            => BinOp::BitAnd,
                    BinaryOp::BitOr             => BinOp::BitOr,
                    BinaryOp::BitXor            => BinOp::BitXor,
                    BinaryOp::LShift            => BinOp::Shl,
                    BinaryOp::RShift            => BinOp::Shr,
                    BinaryOp::ZeroFillRShift    => BinOp::UShr,
                    _ => BinOp::Add,
                };
                let ty = match &op {
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le
                    | BinOp::Gt | BinOp::Ge | BinOp::In => Type::Bool,
                    BinOp::Add | BinOp::NullishCoalesce => Type::Any,
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor
                    | BinOp::Shl | BinOp::Shr | BinOp::UShr => Type::Int,
                    _ => Type::Double,
                };
                HirExpr::BinOp(op, Box::new(left), Box::new(right), ty)
            }
        }
    }
}
