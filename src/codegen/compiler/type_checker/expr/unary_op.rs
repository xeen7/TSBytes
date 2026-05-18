use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_unary_op(&mut self, op: &UnaryOp, arg: &HirExpr) -> Type {
        let arg_ty = self.check_expr(arg);
        match op {
            UnaryOp::Not => Type::Bool,
            UnaryOp::Neg | UnaryOp::Pos => {
                if !matches!(arg_ty, Type::Int | Type::Double | Type::Any) {
                    self.errors.push(format!("Type error: Unary math operator requires numeric type, got '{:?}'", arg_ty));
                }
                Type::Double
            }
            UnaryOp::TypeOf => Type::StringTy,
            UnaryOp::VoidOp => Type::Void,
            _ => Type::Any,
        }
    }
}
