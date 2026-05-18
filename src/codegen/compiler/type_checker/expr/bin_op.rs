use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_bin_op(&mut self, op: &BinOp, left: &HirExpr, right: &HirExpr) -> Type {
        let l_ty = self.check_expr(left);
        let r_ty = self.check_expr(right);
        
        let is_dynamic = matches!(l_ty, Type::Any) || matches!(r_ty, Type::Any)
                      || matches!(l_ty, Type::StringTy) || matches!(r_ty, Type::StringTy)
                      || matches!(l_ty, Type::Union(_)) || matches!(r_ty, Type::Union(_));

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Exp => {
                if *op == BinOp::Add {
                    let l_ok = l_ty.is_primitive() || matches!(l_ty, Type::StringTy) || matches!(l_ty, Type::Any);
                    let r_ok = r_ty.is_primitive() || matches!(r_ty, Type::StringTy) || matches!(r_ty, Type::Any);
                    if !l_ok || !r_ok {
                        self.errors.push(format!("Type error: Arithmetic operations require numeric types, got '{:?}' and '{:?}'", l_ty, r_ty));
                    }
                } else {
                    let l_ok = l_ty.is_primitive() || matches!(l_ty, Type::Any);
                    let r_ok = r_ty.is_primitive() || matches!(r_ty, Type::Any);
                    if !l_ok || !r_ok {
                        self.errors.push(format!("Type error: Arithmetic operations require numeric types, got '{:?}' and '{:?}'", l_ty, r_ty));
                    }
                }
            }
            _ => {}
        }
        
        match op {
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::In => Type::Bool,
            _ => {
                if is_dynamic && *op == BinOp::Add {
                    if matches!(l_ty, Type::StringTy) || matches!(r_ty, Type::StringTy) {
                        Type::StringTy
                    } else {
                        Type::Any
                    }
                } else if is_dynamic {
                    Type::Double
                } else if l_ty == Type::Double || r_ty == Type::Double { Type::Double }
                else if l_ty == Type::Int && r_ty == Type::Int { Type::Int }
                else { Type::Any }
            }
        }
    }
}
