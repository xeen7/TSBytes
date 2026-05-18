pub mod literal;
pub mod ident;
pub mod bin;
pub mod unary;
pub mod assign;
pub mod call;
pub mod member;
pub mod opt_chain;
pub mod new;
pub mod array;
pub mod object;
pub mod ts_as;
pub mod arrow;
pub mod cond;
pub mod tpl;
pub mod update;
pub mod paren;
pub mod this;
pub mod await_expr;
pub mod fn_expr;
pub mod class_expr;

use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use super::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_expr(&mut self, expr: &Expr) -> HirExpr {
        match expr {
            Expr::Lit(lit) => self.build_lit_expr(lit),
            Expr::Ident(ident) => self.build_ident_expr(ident),
            Expr::Bin(bin) => self.build_bin_expr(bin),
            Expr::Unary(unary) => self.build_unary_expr(unary),
            Expr::Assign(assign) => self.build_assign_expr(assign),
            Expr::Call(call) => self.build_call_expr(call),
            Expr::Member(member) => self.build_member_expr(member),
            Expr::OptChain(opt_chain) => self.build_opt_chain_expr(opt_chain),
            Expr::New(new_expr) => self.build_new_expr(new_expr),
            Expr::Array(arr) => self.build_array_lit_expr(arr),
            Expr::Object(obj) => self.build_object_lit_expr(obj),
            Expr::TsAs(ts_as) => self.build_ts_as_expr(ts_as),
            Expr::Arrow(arrow) => self.build_arrow_expr(arrow),
            Expr::Cond(cond) => self.build_cond_expr(cond),
            Expr::Tpl(tpl) => self.build_tpl_expr(tpl),
            Expr::TaggedTpl(tagged_tpl) => self.build_tagged_tpl_expr(tagged_tpl),
            Expr::Yield(yield_expr) => self.build_yield_expr(yield_expr),
            Expr::Update(update) => self.build_update_expr(update),
            Expr::Paren(paren) => self.build_paren_expr(paren),
            Expr::This(this_expr) => self.build_this_expr(this_expr),
            Expr::Await(await_expr) => self.build_await_expr(await_expr),
            Expr::Fn(fn_expr) => self.build_fn_expr(fn_expr),
            Expr::SuperProp(_) => HirExpr::Var("super".to_string(), Type::Any),
            Expr::Class(class_expr) => self.build_class_expr(class_expr),
            Expr::MetaProp(meta) => {
                match meta.kind {
                    MetaPropKind::NewTarget => {
                        if let Some(class_name) = &self.current_class {
                            HirExpr::Var(class_name.clone(), Type::Class(class_name.clone()))
                        } else {
                            HirExpr::UndefinedLit
                        }
                    }
                    MetaPropKind::ImportMeta => {
                        let url_key = "url".to_string();
                        let url_val = HirExpr::StringLit("file:///input.tsx".to_string());
                        let props = vec![ObjectProp::KeyValue(url_key.clone(), url_val)];
                        let ty = Type::Object(vec![(url_key, Type::StringTy)]);
                        HirExpr::ObjectLit(props, ty)
                    }
                }
            }
            Expr::TsNonNull(non_null) => self.build_expr(&non_null.expr),
            Expr::TsSatisfies(sat) => self.build_expr(&sat.expr),
            Expr::TsConstAssertion(c) => self.build_expr(&c.expr),
            Expr::Seq(seq) => {
                let exprs: Vec<HirExpr> = seq.exprs.iter().map(|e| self.build_expr(e)).collect();
                let last_ty = exprs.last().map(|e| e.get_type()).unwrap_or(Type::Any);
                HirExpr::Seq(exprs, last_ty)
            }
            _ => HirExpr::UndefinedLit,
        }
    }
}
