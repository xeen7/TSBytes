use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_interface_decl(&mut self, name: &str) {
        self.env.bind(name.to_string(), Type::Class(name.to_string()));
    }
}
