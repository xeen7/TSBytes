use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_arrow_expr(&mut self, arrow: &ArrowExpr) -> HirExpr {
        let mut args = Vec::new();
        let mut prepended_stmts = Vec::new();
        for (i, param) in arrow.params.iter().enumerate() {
            match param {
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
        let ret_type = self.extract_type(arrow.return_type.as_deref());
        let mut body_stmts = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => self.build_block_stmt(block),
            BlockStmtOrExpr::Expr(expr) => {
                vec![HirStmt::Return(Some(self.build_expr(expr)))]
            }
        };
        body_stmts.splice(0..0, prepended_stmts);
        let modifiers = FnModifiers {
            is_async: arrow.is_async,
            is_generator: false,
            is_export: false,
        };
        HirExpr::Arrow(args, ret_type, body_stmts, modifiers)
    }
}
