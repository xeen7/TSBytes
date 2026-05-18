use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_call(&mut self, name: &str, args: &[HirExpr]) -> Type {
        let mut arg_types = Vec::new();
        for arg in args {
            arg_types.push(self.check_expr(arg));
        }
        
        let fn_ty = self.env.get_type(name);
        if let Type::Function(expected_args, ret_ty) = fn_ty {
            if expected_args.len() != arg_types.len() {
                self.errors.push(format!("Type error: Expected {} arguments for '{}', but got {}", expected_args.len(), name, arg_types.len()));
            } else {
                for (i, (expected, actual)) in expected_args.iter().zip(arg_types.iter()).enumerate() {
                    if !actual.is_assignable_to(expected) {
                        self.errors.push(format!("Type error: Argument {} to '{}' is not assignable to '{:?}', got '{:?}'", i, name, expected, actual));
                    }
                }
            }
            *ret_ty
        } else if fn_ty != Type::Any {
            self.errors.push(format!("Type error: Cannot invoke non-function '{:?}'", fn_ty));
            Type::Any
        } else {
            Type::Any
        }
    }
}
