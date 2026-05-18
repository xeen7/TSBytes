use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_env::TypeEnv;

use std::collections::HashMap;

pub mod expr;
pub mod stmt;

pub struct TypeChecker {
    pub(crate) env: TypeEnv,
    pub(crate) errors: Vec<String>,
    pub(crate) current_fn_return: Option<Type>,
    pub(crate) classes: HashMap<String, ClassDef>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            errors: Vec::new(),
            current_fn_return: None,
            classes: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, stmts: &[HirStmt]) -> Vec<String> {
        self.errors.clear();
        for stmt in stmts {
            self.check_stmt(stmt);
        }
        self.errors.clone()
    }
}
