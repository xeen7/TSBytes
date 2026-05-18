use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_arrow(&mut self, args: &[FnArg], ret_type: &Type, body: &[HirStmt]) -> Type {
        self.env.enter_scope();
        for arg in args {
            self.env.bind(arg.name.clone(), arg.ty.clone());
        }
        let prev_ret = self.current_fn_return.clone();
        self.current_fn_return = Some(ret_type.clone());
        
        for s in body { self.check_stmt(s); }
        
        self.current_fn_return = prev_ret;
        self.env.exit_scope();
        
        Type::Function(
            args.iter().map(|a| a.ty.clone()).collect(),
            Box::new(ret_type.clone())
        )
    }
}
