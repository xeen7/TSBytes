use crate::codegen::compiler::type_checker::TypeChecker;
use crate::codegen::compiler::ir::*;

impl TypeChecker {
    pub(crate) fn check_update(&mut self, _name: &str) -> Type {
        Type::Double
    }
}
