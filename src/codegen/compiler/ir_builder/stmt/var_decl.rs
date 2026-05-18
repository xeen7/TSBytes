use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_var_decl(&mut self, var_decl: &VarDecl) -> Option<HirStmt> {
        if var_decl.declare {
            if let Some(decl) = var_decl.decls.first() {
                if let Pat::Ident(ident) = &decl.name {
                    let name = ident.id.sym.to_string();
                    let explicit_type = self.extract_type(ident.type_ann.as_deref());
                    self.env.bind(name, explicit_type);
                }
            }
            return None;
        }
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
                return Some(HirStmt::Let(name, ty, expr));
            } else {
                let source = if let Some(init) = &decl.init {
                    self.build_expr(init)
                } else {
                    HirExpr::UndefinedLit
                };
                return self.build_destructuring_stmt(&decl.name, source);
            }
        }
        None
    }

    pub(crate) fn build_destructuring_stmt(&mut self, pat: &Pat, source: HirExpr) -> Option<HirStmt> {
        let stmts = self.flatten_destructuring(pat, source);
        if stmts.is_empty() {
            None
        } else {
            Some(HirStmt::If(HirExpr::BoolLit(true), stmts, vec![]))
        }
    }

    pub(crate) fn flatten_destructuring(&mut self, pat: &Pat, source: HirExpr) -> Vec<HirStmt> {
        match pat {
            Pat::Ident(ident) => {
                let name = ident.id.sym.to_string();
                let explicit_type = self.extract_type(ident.type_ann.as_deref());
                let ty = if explicit_type != Type::Any { explicit_type } else { source.get_type() };
                self.env.bind(name.clone(), ty.clone());
                vec![HirStmt::Let(name, ty, source)]
            }
            Pat::Object(obj_pat) => {
                let temp_name = self.next_temp_name();
                self.env.bind(temp_name.clone(), Type::Any);
                let temp_var = HirExpr::Var(temp_name.clone(), Type::Any);

                let mut stmts = vec![HirStmt::Let(temp_name, Type::Any, source)];
                let mut flat_fields = Vec::new();
                let mut rest_name = None;

                for prop in &obj_pat.props {
                    match prop {
                        swc_core::ecma::ast::ObjectPatProp::KeyValue(kv) => {
                            let key = if let PropName::Ident(id) = &kv.key {
                                id.sym.to_string()
                            } else { continue };

                            match &*kv.value {
                                Pat::Ident(id) => {
                                    let name = id.id.sym.to_string();
                                    self.env.bind(name.clone(), Type::Any);
                                    flat_fields.push((key, Some(name), None));
                                }
                                nested_pat => {
                                    let field_temp = self.next_temp_name();
                                    self.env.bind(field_temp.clone(), Type::Any);
                                    flat_fields.push((key, Some(field_temp.clone()), None));

                                    let field_expr = HirExpr::Var(field_temp, Type::Any);
                                    stmts.extend(self.flatten_destructuring(nested_pat, field_expr));
                                }
                            }
                        }
                        swc_core::ecma::ast::ObjectPatProp::Assign(assign) => {
                            let name = assign.key.sym.to_string();
                            let default_val = assign.value.as_ref().map(|v| self.build_expr(v));
                            self.env.bind(name.clone(), Type::Any);
                            flat_fields.push((name, None, default_val));
                        }
                        swc_core::ecma::ast::ObjectPatProp::Rest(rest) => {
                            match &*rest.arg {
                                Pat::Ident(id) => {
                                    let name = id.id.sym.to_string();
                                    self.env.bind(name.clone(), Type::Any);
                                    rest_name = Some(name);
                                }
                                nested_pat => {
                                    let rest_temp = self.next_temp_name();
                                    self.env.bind(rest_temp.clone(), Type::Any);
                                    rest_name = Some(rest_temp.clone());

                                    let rest_expr = HirExpr::Var(rest_temp, Type::Any);
                                    stmts.extend(self.flatten_destructuring(nested_pat, rest_expr));
                                }
                            }
                        }
                    }
                }
                stmts.insert(1, HirStmt::DestructureObject(flat_fields, rest_name, temp_var));
                stmts
            }
            Pat::Array(arr_pat) => {
                let source_ty = source.get_type();
                let elem_ty = match &source_ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::Any,
                };
                let rest_ty = Type::Array(Box::new(elem_ty.clone()));

                let temp_name = self.next_temp_name();
                let temp_ty = if source_ty != Type::Any { source_ty.clone() } else { Type::Any };
                self.env.bind(temp_name.clone(), temp_ty.clone());
                let temp_var = HirExpr::Var(temp_name.clone(), temp_ty.clone());

                let mut stmts = vec![HirStmt::Let(temp_name, temp_ty, source)];
                let mut flat_slots = Vec::new();
                let mut rest_name = None;

                for elem in &arr_pat.elems {
                    if let Some(pat) = elem {
                        match pat {
                            Pat::Ident(id) => {
                                let name = id.id.sym.to_string();
                                self.env.bind(name.clone(), elem_ty.clone());
                                flat_slots.push((Some(name), None));
                            }
                            Pat::Assign(assign) => {
                                if let Pat::Ident(id) = &*assign.left {
                                    let name = id.id.sym.to_string();
                                    let default_val = Some(self.build_expr(&assign.right));
                                    self.env.bind(name.clone(), elem_ty.clone());
                                    flat_slots.push((Some(name), default_val));
                                } else {
                                    let elem_temp = self.next_temp_name();
                                    self.env.bind(elem_temp.clone(), elem_ty.clone());
                                    let default_val = Some(self.build_expr(&assign.right));
                                    flat_slots.push((Some(elem_temp.clone()), default_val));

                                    let elem_expr = HirExpr::Var(elem_temp, elem_ty.clone());
                                    stmts.extend(self.flatten_destructuring(&assign.left, elem_expr));
                                }
                            }
                            Pat::Rest(rest) => {
                                match &*rest.arg {
                                    Pat::Ident(id) => {
                                        let name = id.id.sym.to_string();
                                        self.env.bind(name.clone(), rest_ty.clone());
                                        rest_name = Some(name);
                                    }
                                    nested_pat => {
                                        let rest_temp = self.next_temp_name();
                                        self.env.bind(rest_temp.clone(), rest_ty.clone());
                                        rest_name = Some(rest_temp.clone());

                                        let rest_expr = HirExpr::Var(rest_temp, rest_ty.clone());
                                        stmts.extend(self.flatten_destructuring(nested_pat, rest_expr));
                                    }
                                }
                            }
                            nested_pat => {
                                let elem_temp = self.next_temp_name();
                                self.env.bind(elem_temp.clone(), elem_ty.clone());
                                flat_slots.push((Some(elem_temp.clone()), None));

                                let elem_expr = HirExpr::Var(elem_temp, elem_ty.clone());
                                stmts.extend(self.flatten_destructuring(nested_pat, elem_expr));
                            }
                        }
                    } else {
                        flat_slots.push((None, None));
                    }
                }
                stmts.insert(1, HirStmt::DestructureArray(flat_slots, rest_name, temp_var));
                stmts
            }
            Pat::Assign(assign) => {
                let temp_name = self.next_temp_name();
                self.env.bind(temp_name.clone(), Type::Any);
                let default_expr = self.build_expr(&assign.right);
                
                let cond = HirExpr::BinOp(
                    BinOp::Eq,
                    Box::new(source.clone()),
                    Box::new(HirExpr::NullLit),
                    Type::Bool,
                );
                let fallback = HirExpr::Ternary(
                    Box::new(cond),
                    Box::new(default_expr),
                    Box::new(source),
                    Type::Any,
                );
                
                let mut stmts = vec![HirStmt::Let(temp_name.clone(), Type::Any, fallback)];
                let temp_var = HirExpr::Var(temp_name, Type::Any);
                stmts.extend(self.flatten_destructuring(&assign.left, temp_var));
                stmts
            }
            _ => vec![],
        }
    }
}
