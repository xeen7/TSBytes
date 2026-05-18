use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_destructure_object(&mut self, fields: &[(String, Option<String>, Option<HirExpr>)], source: &HirExpr) {
        self.check_expr(source);
        for (prop_name, alias, default) in fields {
            let local = alias.as_deref().unwrap_or(prop_name);
            if let Some(d) = default { self.check_expr(d); }
            self.env.bind(local.to_string(), Type::Any);
        }
    }

    pub(crate) fn check_destructure_array(&mut self, slots: &[(Option<String>, Option<HirExpr>)], source: &HirExpr) {
        self.check_expr(source);
        for (name_opt, default) in slots {
            if let Some(name) = name_opt {
                if let Some(d) = default { self.check_expr(d); }
                self.env.bind(name.to_string(), Type::Any);
            }
        }
    }
}
