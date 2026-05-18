use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_var(&mut self, name: &str) -> Type {
        self.env.get_type(name)
    }
}
