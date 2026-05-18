use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_this(&mut self, ty: &Type) -> Type {
        ty.clone()
    }
}
