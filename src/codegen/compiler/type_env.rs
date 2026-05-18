use std::collections::HashMap;
use crate::codegen::compiler::ir::Type;

pub struct TypeEnv {
    bindings: HashMap<String, Type>,
    scopes: Vec<HashMap<String, Type>>,
    custom_types: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            scopes: vec![HashMap::new()],
            custom_types: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn bind(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), ty.clone());
        }
        self.bindings.insert(name, ty);
    }

    pub fn get_type(&self, name: &str) -> Type {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return ty.clone();
            }
        }
        Type::Any
    }

    pub fn register_type(&mut self, name: String, ty: Type) {
        self.custom_types.insert(name, ty);
    }

    pub fn resolve_type(&self, name: &str) -> Type {
        if let Some(ty) = self.custom_types.get(name) {
            ty.clone()
        } else {
            Type::Any
        }
    }
}
