use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_fn_expr(&mut self, fn_expr: &FnExpr) -> HirExpr {
        let mut args = Vec::new();
        let mut prepended_stmts = Vec::new();
        
        for (i, param) in fn_expr.function.params.iter().enumerate() {
            match &param.pat {
                Pat::Ident(binding) => {
                    let arg_name = binding.id.sym.to_string();
                    let arg_type = self.extract_type(binding.type_ann.as_deref());
                    args.push(FnArg { name: arg_name.clone(), ty: arg_type.clone(), is_rest: false });
                    self.env.bind(arg_name, arg_type);
                }
                Pat::Rest(rest) => {
                    if let Pat::Ident(binding) = &*rest.arg {
                        let arg_name = binding.id.sym.to_string();
                        let arg_type = self.extract_type(rest.type_ann.as_deref());
                        args.push(FnArg { name: arg_name.clone(), ty: arg_type.clone(), is_rest: true });
                        self.env.bind(arg_name, arg_type);
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
        
        let ret_type = self.extract_type(fn_expr.function.return_type.as_deref());
        let mut body_stmts = fn_expr.function.body.as_ref()
            .map(|body| self.build_block_stmt(body))
            .unwrap_or_else(Vec::new);
            
        body_stmts.splice(0..0, prepended_stmts);
        
        let modifiers = FnModifiers {
            is_async: fn_expr.function.is_async,
            is_generator: fn_expr.function.is_generator,
            is_export: false,
        };
        
        HirExpr::Arrow(args, ret_type, body_stmts, modifiers)
    }
}
