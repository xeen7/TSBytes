use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_for_of(&mut self, name: &str, ty: &Type, iterable: &HirExpr, body: &[HirStmt]) {
        let iter_ty = self.check_expr(iterable);
        if let Type::Array(inner) = iter_ty {
            if !inner.is_assignable_to(ty) {
                self.errors.push(format!("Type error: Iterable element type '{:?}' is not assignable to loop variable type '{:?}'", inner, ty));
            }
        }
        self.env.enter_scope();
        self.env.bind(name.to_string(), ty.clone());
        for s in body { self.check_stmt(s); }
        self.env.exit_scope();
    }

    pub(crate) fn check_for_in(&mut self, name: &str, obj: &HirExpr, body: &[HirStmt]) {
        self.check_expr(obj);
        self.env.enter_scope();
        self.env.bind(name.to_string(), Type::StringTy);
        for s in body { self.check_stmt(s); }
        self.env.exit_scope();
    }
}
