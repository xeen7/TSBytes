use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_lit_expr(&mut self, lit: &Lit) -> HirExpr {
        match lit {
            Lit::Num(n) => HirExpr::DoubleLit(n.value),
            Lit::Str(s) => HirExpr::StringLit(s.value.to_string()),
            Lit::Bool(b) => HirExpr::BoolLit(b.value),
            Lit::Null(_) => HirExpr::NullLit,
            Lit::BigInt(b) => HirExpr::BigIntLit(b.value.to_string()),
            Lit::Regex(r) => HirExpr::RegExpLit(r.exp.to_string(), r.flags.to_string()),
            _ => HirExpr::UndefinedLit,
        }
    }
}
