use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_array_lit(&mut self, elems: &[HirExpr]) -> Type {
        let mut elem_type = Type::Any;
        for elem in elems {
            let ety = self.check_expr(elem);
            if elem_type == Type::Any { elem_type = ety; }
        }
        Type::Array(Box::new(elem_type))
    }
}
