use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_opt_chain_expr(&mut self, opt_chain: &OptChainExpr) -> HirExpr {
        match &*opt_chain.base {
            OptChainBase::Member(member) => {
                let obj = self.build_expr(&member.obj);
                if let MemberProp::Ident(ident) = &member.prop {
                    let prop_name = ident.sym.to_string();
                    HirExpr::OptionalChain(Box::new(obj), prop_name, Type::Any)
                } else {
                    HirExpr::UndefinedLit
                }
            }
            OptChainBase::Call(opt_call) => {
                let callee = self.build_expr(&opt_call.callee);
                let mut args = Vec::new();
                for arg in &opt_call.args {
                    args.push(self.build_expr(&arg.expr));
                }
                HirExpr::OptionalCall(Box::new(callee), args, Type::Any)
            }
        }
    }
}
