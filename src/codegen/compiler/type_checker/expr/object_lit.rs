use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_object_lit(&mut self, props: &[ObjectProp]) -> Type {
        let mut obj_props = Vec::new();
        for prop in props {
            match prop {
                ObjectProp::KeyValue(name, val) => {
                    let val_ty = self.check_expr(val);
                    obj_props.push((name.clone(), val_ty));
                }
                ObjectProp::Computed(key, val) => {
                    self.check_expr(key);
                    self.check_expr(val);
                }
                ObjectProp::Method(name, _args, ret_ty, _body) => {
                    obj_props.push((name.clone(), ret_ty.clone()));
                }
                ObjectProp::Getter(name, _body) => {
                    obj_props.push((name.clone(), Type::Any));
                }
                ObjectProp::Setter(name, _expr) => {
                    obj_props.push((name.clone(), Type::Any));
                }
                ObjectProp::Spread(expr) => {
                    self.check_expr(expr);
                }
            }
        }
        Type::Object(obj_props)
    }
}
