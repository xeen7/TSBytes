use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_literal(&mut self, expr: &HirExpr) -> Type {
        match expr {
            HirExpr::IntLit(_) => Type::Int,
            HirExpr::DoubleLit(_) => Type::Double,
            HirExpr::StringLit(_) => Type::StringTy,
            HirExpr::BoolLit(_) => Type::Bool,
            HirExpr::NullLit => Type::Null,
            HirExpr::UndefinedLit => Type::Undefined,
            HirExpr::BigIntLit(_) => Type::BigInt,
            HirExpr::RegExpLit(_, _) => Type::RegExp,
            _ => Type::Any,
        }
    }
}
