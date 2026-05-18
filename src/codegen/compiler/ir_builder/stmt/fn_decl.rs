use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_fn_decl(&mut self, fn_decl: &FnDecl) -> Option<HirStmt> {
        if fn_decl.declare {
            let name = fn_decl.ident.sym.to_string();
            let args: Vec<Type> = fn_decl.function.params.iter().map(|p| {
                if let Pat::Ident(binding) = &p.pat {
                    self.extract_type(binding.type_ann.as_deref())
                } else {
                    Type::Any
                }
            }).collect();
            let ret = self.extract_type(fn_decl.function.return_type.as_deref());
            self.env.bind(name, Type::Function(args, Box::new(ret)));
            return None;
        }
        let name = fn_decl.ident.sym.to_string();
        let mut args = Vec::new();
        let mut prepended_stmts = Vec::new();
        for (i, param) in fn_decl.function.params.iter().enumerate() {
            match &param.pat {
                Pat::Ident(binding) => {
                    let arg_name = binding.id.sym.to_string();
                    let arg_type = self.extract_type(binding.type_ann.as_deref());
                    args.push(FnArg { name: arg_name, ty: arg_type, is_rest: false });
                }
                Pat::Rest(rest) => {
                    if let Pat::Ident(binding) = &*rest.arg {
                        let arg_name = binding.id.sym.to_string();
                        // The type annotation is typically on the Rest pattern itself, not the inner Ident
                        let arg_type = self.extract_type(rest.type_ann.as_deref());
                        args.push(FnArg { name: arg_name, ty: arg_type, is_rest: true });
                    }
                }
                other_pat => {
                    let synth_name = format!("_param_{}", i);
                    args.push(FnArg { name: synth_name.clone(), ty: Type::Any, is_rest: false });
                    if let Some(destruct) = self.build_destructuring_stmt(other_pat, HirExpr::Var(synth_name, Type::Any)) {
                        prepended_stmts.push(destruct);
                    }
                }
            }
        }
        let ret_type = self.extract_type(fn_decl.function.return_type.as_deref());
        let fn_type = Type::Function(
            args.iter().map(|arg| arg.ty.clone()).collect(),
            Box::new(ret_type.clone())
        );
        self.env.bind(name.clone(), fn_type);
        
        self.env.enter_scope();
        for arg in &args {
            self.env.bind(arg.name.clone(), arg.ty.clone());
        }
        
        let mut body_stmts = self.build_block_stmt(&fn_decl.function.body.as_ref().unwrap());
        body_stmts.splice(0..0, prepended_stmts);
        self.env.exit_scope();
        
        let mut modifiers = FnModifiers {
            is_async: fn_decl.function.is_async,
            is_generator: fn_decl.function.is_generator,
            is_export: false,
        };

        if fn_decl.function.is_async && !fn_decl.function.is_generator && name != "main" {
            // Turn off is_async for the generated JVM method, since its body does the return synchronously
            modifiers.is_async = false;
            let closure = HirExpr::Arrow(vec![], ret_type.clone(), body_stmts, FnModifiers { is_async: false, is_generator: false, is_export: false });
            let spawn = HirExpr::VirtualThreadSpawn(Box::new(closure));
            body_stmts = vec![HirStmt::Return(Some(spawn))];
        }
        
        Some(HirStmt::FnDecl(name, args, ret_type, body_stmts, modifiers))
    }
}
