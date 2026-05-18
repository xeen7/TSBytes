pub mod decl;
pub mod assign;
pub mod expr_stmt;
pub mod return_stmt;
pub mod if_stmt;
pub mod while_stmt;
pub mod control_flow;
pub mod try_catch;
pub mod throw_stmt;
pub mod for_loop;
pub mod destructure;

use crate::codegen::compiler::ir::*;
use super::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_stmts(&mut self, stmts: &[HirStmt]) {
        for s in stmts {
            self.emit_stmt(s);
            if matches!(s, HirStmt::Return(..) | HirStmt::Throw(..) | HirStmt::Break(..) | HirStmt::Continue(..)) {
                break;
            }
        }
    }

    pub(crate) fn emit_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let(..) | HirStmt::Const(..) => self.emit_decl(stmt),
            HirStmt::Assign(..) | HirStmt::CompoundAssign(..) | HirStmt::FieldAssign(..) => self.emit_assign(stmt),
            HirStmt::Expr(..) => self.emit_expr_stmt(stmt),
            HirStmt::Return(..) => self.emit_return_stmt(stmt),
            HirStmt::If(..) => self.emit_if_stmt(stmt),
            HirStmt::While(..) | HirStmt::DoWhile(..) => self.emit_while_stmt(stmt),
            HirStmt::Break(..) | HirStmt::Continue(..) | HirStmt::Import(..) | HirStmt::FnDecl(..) | HirStmt::Switch(..) | HirStmt::InterfaceDecl(..) | HirStmt::Export(..) | HirStmt::Labeled(..) | HirStmt::ClassDecl(..) => self.emit_control_flow(stmt),
            HirStmt::TryCatch(..) => self.emit_try_catch(stmt),
            HirStmt::Throw(..) => self.emit_throw_stmt(stmt),
            HirStmt::ForOf(..) | HirStmt::ForAwaitOf(..) | HirStmt::ForIn(..) => self.emit_for_loop(stmt),
            HirStmt::DestructureObject(..) | HirStmt::DestructureArray(..) => self.emit_destructure(stmt),
        }
    }
}
