use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_field_get(&mut self, obj: &HirExpr, prop_name: &str) -> Type {
        let obj_ty = self.check_expr(obj);
        if let Type::Object(props) = obj_ty {
            if let Some((_, ty)) = props.iter().find(|(k, _)| k == prop_name) {
                return ty.clone();
            }
            self.errors.push(format!("Type error: Property '{}' does not exist on type '{:?}'", prop_name, Type::Object(props)));
        }
        Type::Any
    }

    pub(crate) fn check_field_set(&mut self, obj: &HirExpr, prop_name: &str, val: &HirExpr) -> Type {
        let obj_ty = self.check_expr(obj);
        let val_ty = self.check_expr(val);
        if let Type::Object(props) = obj_ty {
            if let Some((_, ty)) = props.iter().find(|(k, _)| k == prop_name) {
                if !val_ty.is_assignable_to(ty) {
                    self.errors.push(format!("Type error: Cannot assign type '{:?}' to property '{}' of type '{:?}'", val_ty, prop_name, ty));
                }
            } else {
                self.errors.push(format!("Type error: Property '{}' does not exist on type '{:?}'", prop_name, Type::Object(props)));
            }
        }
        Type::Any
    }
}
