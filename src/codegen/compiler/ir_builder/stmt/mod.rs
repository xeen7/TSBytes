pub mod var_decl;
pub mod expr_stmt;
pub mod return_stmt;
pub mod if_stmt;
pub mod while_stmt;
pub mod for_stmt;
pub mod switch_stmt;
pub mod try_stmt;
pub mod throw_stmt;
pub mod break_stmt;
pub mod continue_stmt;
pub mod block_stmt;
pub mod fn_decl;
pub mod class_decl;
pub mod ts_interface_decl;
pub mod ts_type_alias_decl;
pub mod for_of_stmt;
pub mod do_while_stmt;
pub mod labeled_stmt;
pub mod ts_enum_decl;
pub mod ts_module_decl;
pub mod for_in_stmt;

use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use super::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_stmt(&mut self, stmt: &Stmt) -> Option<HirStmt> {
        match stmt {
            Stmt::Decl(Decl::Var(var_decl)) => self.build_var_decl(var_decl),
            Stmt::Expr(expr_stmt) => self.build_expr_stmt(expr_stmt),
            Stmt::Return(ret_stmt) => self.build_return_stmt(ret_stmt),
            Stmt::If(if_stmt) => self.build_if_stmt(if_stmt),
            Stmt::While(while_stmt) => self.build_while_stmt(while_stmt),
            Stmt::For(for_stmt) => self.build_for_stmt(for_stmt),
            Stmt::Switch(switch_stmt) => self.build_switch_stmt(switch_stmt),
            Stmt::Try(try_stmt) => self.build_try_stmt(try_stmt),
            Stmt::Throw(throw_stmt) => self.build_throw_stmt(throw_stmt),
            Stmt::Break(break_stmt) => self.build_break_stmt(break_stmt),
            Stmt::Continue(continue_stmt) => self.build_continue_stmt(continue_stmt),
            Stmt::Block(block) => self.build_block(block),
            Stmt::Decl(Decl::Fn(fn_decl)) => self.build_fn_decl(fn_decl),
            Stmt::Decl(Decl::Class(class_decl)) => self.build_class_decl(class_decl),
            Stmt::Decl(Decl::TsInterface(iface)) => self.build_ts_interface_decl(iface),
            Stmt::Decl(Decl::TsTypeAlias(alias)) => self.build_ts_type_alias_decl(alias),
            Stmt::Decl(Decl::TsEnum(enum_decl)) => self.build_ts_enum_decl(enum_decl),
            Stmt::Decl(Decl::TsModule(module_decl)) => self.build_ts_module_decl(module_decl),
            Stmt::ForOf(for_of) => self.build_for_of_stmt(for_of),
            Stmt::ForIn(for_in) => self.build_for_in_stmt(for_in),
            Stmt::DoWhile(do_while) => self.build_do_while_stmt(do_while),
            Stmt::Labeled(labeled) => self.build_labeled_stmt(labeled),
            _ => None,
        }
    }

    pub(crate) fn build_block_stmt(&mut self, block: &BlockStmt) -> Vec<HirStmt> {
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            if let Some(hir_stmt) = self.build_stmt(stmt) {
                stmts.push(hir_stmt);
            }
        }
        stmts
    }
}
