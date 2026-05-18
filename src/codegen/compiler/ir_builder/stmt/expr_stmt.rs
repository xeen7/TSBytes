use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> Option<HirStmt> {
        // Intercept top-level and block-level string directives (e.g. "use strict") and erase them cleanly.
        if let Expr::Lit(Lit::Str(_)) = &*expr_stmt.expr {
            return None;
        }
        if let Expr::Assign(assign) = &*expr_stmt.expr {
            let hir_op = swc_assign_op_to_hir(&assign.op);

            // Simple variable target: `x = rhs` or `x += rhs`
            if let AssignTarget::Simple(SimpleAssignTarget::Ident(ident)) = &assign.left {
                let name = ident.id.sym.to_string();
                let right = self.build_expr(&assign.right);
                return Some(match hir_op {
                    crate::codegen::compiler::ir::AssignOp::Assign => HirStmt::Assign(name, right),
                    op => HirStmt::CompoundAssign(name, op, right),
                });
            }

            // Member target: `obj.field = rhs` or `obj[key] = rhs` (no compound)
            if let AssignTarget::Simple(SimpleAssignTarget::Member(member)) = &assign.left {
                let obj = self.build_expr(&member.obj);
                match &member.prop {
                    MemberProp::Ident(ident) => {
                        let prop_name = ident.sym.to_string();
                        let right = self.build_expr(&assign.right);
                        return Some(HirStmt::FieldAssign(obj, prop_name, right));
                    }
                    MemberProp::Computed(computed) => {
                        // obj[key] = val — lower to IndexSet expression statement
                        let key = self.build_expr(&computed.expr);
                        let right = self.build_expr(&assign.right);
                        let ty = right.get_type();
                        return Some(HirStmt::Expr(
                            HirExpr::IndexSet(Box::new(obj), Box::new(key), Box::new(right), ty)
                        ));
                    }
                    _ => {}
                }
            }
        }
        Some(HirStmt::Expr(self.build_expr(&expr_stmt.expr)))
    }
}

/// Convert SWC AssignOp to HIR AssignOp.
fn swc_assign_op_to_hir(op: &swc_core::ecma::ast::AssignOp) -> crate::codegen::compiler::ir::AssignOp {
    use swc_core::ecma::ast::AssignOp as S;
    use crate::codegen::compiler::ir::AssignOp as H;
    match op {
        S::Assign           => H::Assign,
        S::AddAssign        => H::AddAssign,
        S::SubAssign        => H::SubAssign,
        S::MulAssign        => H::MulAssign,
        S::DivAssign        => H::DivAssign,
        S::ModAssign        => H::ModAssign,
        S::ExpAssign        => H::ExpAssign,
        S::BitAndAssign     => H::BitAndAssign,
        S::BitOrAssign      => H::BitOrAssign,
        S::BitXorAssign     => H::BitXorAssign,
        S::LShiftAssign     => H::ShlAssign,
        S::RShiftAssign     => H::ShrAssign,
        S::ZeroFillRShiftAssign => H::UShrAssign,
        S::AndAssign        => H::LogAndAssign,
        S::OrAssign         => H::LogOrAssign,
        S::NullishAssign    => H::NullishAssign,
    }
}
