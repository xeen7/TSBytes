use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_fn_decl(&mut self, name: &str, args: &[FnArg], ret_type: &Type, body: &[HirStmt]) {
        let fn_type = Type::Function(
            args.iter().map(|arg| arg.ty.clone()).collect(),
            Box::new(ret_type.clone())
        );
        self.env.bind(name.to_string(), fn_type);
        
        self.env.enter_scope();
        for arg in args {
            self.env.bind(arg.name.clone(), arg.ty.clone());
        }
        let prev_ret = self.current_fn_return.clone();
        self.current_fn_return = Some(ret_type.clone());
        
        for s in body { self.check_stmt(s); }
        
        self.current_fn_return = prev_ret;
        self.env.exit_scope();
    }
}
