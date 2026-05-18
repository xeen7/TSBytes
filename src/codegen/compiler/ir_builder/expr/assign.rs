use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_assign_expr(&mut self, assign: &AssignExpr) -> HirExpr {
        if let AssignTarget::Simple(SimpleAssignTarget::Ident(_ident)) = &assign.left {
            let right = self.build_expr(&assign.right);
            right // Simple approximation
        } else if let AssignTarget::Simple(SimpleAssignTarget::Member(member)) = &assign.left {
            let obj = self.build_expr(&member.obj);
            match &member.prop {
                MemberProp::Ident(ident) => {
                    let prop_name = ident.sym.to_string();
                    let right = self.build_expr(&assign.right);
                    let ty = right.get_type();
                    HirExpr::FieldSet(Box::new(obj), prop_name, Box::new(right), ty)
                }
                MemberProp::PrivateName(p) => {
                    let prop_name = format!("private${}", p.id.sym);
                    let right = self.build_expr(&assign.right);
                    let ty = right.get_type();
                    HirExpr::FieldSet(Box::new(obj), prop_name, Box::new(right), ty)
                }
                MemberProp::Computed(computed) => {
                    let idx = self.build_expr(&computed.expr);
                    let right = self.build_expr(&assign.right);
                    let ty = Type::Any;
                    HirExpr::IndexSet(Box::new(obj), Box::new(idx), Box::new(right), ty)
                }
            }
        } else {
            HirExpr::UndefinedLit
        }
    }
}
