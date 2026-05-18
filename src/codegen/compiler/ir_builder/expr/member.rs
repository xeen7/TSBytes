use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_member_expr(&mut self, member: &MemberExpr) -> HirExpr {
        let obj = self.build_expr(&member.obj);
        match &member.prop {
            MemberProp::Ident(ident) => {
                let prop_name = ident.sym.to_string();
                if prop_name == "length" && matches!(obj.get_type(), Type::Array(_)) {
                    return HirExpr::ArrayLen(Box::new(obj));
                }
                let ret_ty = if let Type::Object(fields) = obj.get_type() {
                    fields.iter()
                        .find(|(name, _)| name == &prop_name)
                        .map(|(_, ty)| ty.clone())
                        .unwrap_or(Type::Any)
                } else {
                    Type::Any
                };
                HirExpr::FieldGet(Box::new(obj), prop_name, ret_ty)
            }
            MemberProp::PrivateName(p) => {
                let prop_name = format!("private${}", p.id.sym);
                HirExpr::FieldGet(Box::new(obj), prop_name, Type::Any)
            }
            MemberProp::Computed(computed) => {
                let idx = self.build_expr(&computed.expr);
                HirExpr::IndexGet(Box::new(obj), Box::new(idx), Type::Any)
            }
        }
    }
}
