pub mod decl;
pub mod assign;
pub mod return_stmt;
pub mod if_stmt;
pub mod while_stmt;
pub mod for_loop;
pub mod switch;
pub mod control_flow;
pub mod try_catch;
pub mod throw_stmt;
pub mod expr_stmt;
pub mod fn_decl;
pub mod class_decl;
pub mod interface_decl;
pub mod module;
pub mod destructure;

use crate::codegen::compiler::ir::*;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let(name, ty, expr) | HirStmt::Const(name, ty, expr) => self.check_decl(name, ty, expr),
            HirStmt::Assign(name, expr) => self.check_assign(name, expr),
            HirStmt::CompoundAssign(name, op, expr) => self.check_compound_assign(name, op, expr),
            HirStmt::FieldAssign(obj, field, expr) => self.check_field_assign(obj, field, expr),
            HirStmt::Return(expr_opt) => self.check_return(expr_opt),
            HirStmt::If(test, cons, alt) => self.check_if(test, cons, alt),
            HirStmt::While(test, body) => self.check_while(test, body),
            HirStmt::DoWhile(body, test) => self.check_do_while(body, test),
            HirStmt::ForOf(name, ty, iterable, body) | HirStmt::ForAwaitOf(name, ty, iterable, body) => self.check_for_of(name, ty, iterable, body),
            HirStmt::ForIn(name, obj, body) => self.check_for_in(name, obj, body),
            HirStmt::Switch(expr, cases) => self.check_switch(expr, cases),
            HirStmt::Break(_) | HirStmt::Continue(_) => self.check_control_flow(stmt),
            HirStmt::Labeled(label, s) => self.check_labeled(label, s),
            HirStmt::TryCatch(try_blk, catch_var, catch_blk, finally_blk) => self.check_try_catch(try_blk, catch_var, catch_blk, finally_blk),
            HirStmt::Throw(expr) => self.check_throw(expr),
            HirStmt::Expr(expr) => self.check_expr_stmt(expr),
            HirStmt::FnDecl(name, args, ret_type, body, _modifiers) => self.check_fn_decl(name, args, ret_type, body),
            HirStmt::ClassDecl(name, class_def) => self.check_class_decl(name, class_def),
            HirStmt::InterfaceDecl(name, _) => self.check_interface_decl(name),
            HirStmt::Export(s) => self.check_export(s),
            HirStmt::Import(bindings, source) => self.check_import(bindings, source),
            HirStmt::DestructureObject(fields, _, source) => self.check_destructure_object(fields, source),
            HirStmt::DestructureArray(slots, _, source) => self.check_destructure_array(slots, source),
        }
    }
}
