use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_for_stmt(&mut self, for_stmt: &ForStmt) -> Option<HirStmt> {
        self.env.enter_scope();
        let mut body_stmts = Vec::new();
        if let Some(VarDeclOrExpr::VarDecl(var_decl)) = &for_stmt.init {
            if let Some(decl) = var_decl.decls.first() {
                if let Pat::Ident(ident) = &decl.name {
                    let name = ident.id.sym.to_string();
                    let explicit_type = self.extract_type(ident.type_ann.as_deref());
                    let expr = if let Some(init) = &decl.init {
                        self.build_expr(init)
                    } else {
                        HirExpr::UndefinedLit
                    };
                    let ty = if explicit_type != Type::Any { explicit_type } else { expr.get_type() };
                    self.env.bind(name.clone(), ty.clone());
                    body_stmts.push(HirStmt::Let(name, ty, expr));
                }
            }
        }
        
        let test = if let Some(test_expr) = &for_stmt.test {
            self.build_expr(test_expr)
        } else {
            HirExpr::BoolLit(true)
        };

        let mut loop_body = self.build_stmt(&for_stmt.body).map(|s| vec![s]).unwrap_or_default();
        if let Some(update) = &for_stmt.update {
            let update_expr = self.build_expr(update);
            
            // Helper to recursively find and rewrite continues targeting the current loop
            fn rewrite_continues(stmts: &mut Vec<HirStmt>, update: &HirExpr) {
                let mut i = 0;
                while i < stmts.len() {
                    match &mut stmts[i] {
                        HirStmt::Continue(_) => {
                            let update_stmt = HirStmt::Expr(update.clone());
                            let cont = stmts[i].clone();
                            stmts[i] = update_stmt;
                            stmts.insert(i + 1, cont);
                            i += 2;
                        }
                        HirStmt::If(_, cons, alt) => {
                            rewrite_continues(cons, update);
                            rewrite_continues(alt, update);
                            i += 1;
                        }
                        // Do not enter inner loops
                        HirStmt::While(_, _) | HirStmt::DoWhile(_, _) | HirStmt::ForOf(_, _, _, _) | HirStmt::ForIn(_, _, _) => {
                            i += 1;
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }
            }
            
            rewrite_continues(&mut loop_body, &update_expr);
            loop_body.push(HirStmt::Expr(update_expr));
        }

        body_stmts.push(HirStmt::While(test, loop_body));
        self.env.exit_scope();
        
        Some(HirStmt::If(HirExpr::BoolLit(true), body_stmts, vec![]))
    }
}
