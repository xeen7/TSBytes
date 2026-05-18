use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_class_decl(&mut self, class_decl: &ClassDecl) -> Option<HirStmt> {
        if class_decl.declare {
            let name = class_decl.ident.sym.to_string();
            self.env.bind(name.clone(), Type::Class(name));
            return None;
        }
        let name = class_decl.ident.sym.to_string();
        let old_class = self.current_class.replace(name.clone());
        let mut members = Vec::new();

        use swc_core::ecma::ast::ClassMember as SwcClassMember;
        use crate::codegen::compiler::ir::ClassMember as HirClassMember;

        for member in &class_decl.class.body {
            match member {
                // ── Instance / static fields ─────────────────────────────────────────
                SwcClassMember::ClassProp(prop) => {
                    if let PropName::Ident(ident) = &prop.key {
                        let field_name = ident.sym.to_string();
                        let ty = self.extract_type(prop.type_ann.as_deref());
                        let init = prop.value.as_ref().map(|v| self.build_expr(v));
                        let mods = FieldModifiers {
                            is_static:    prop.is_static,
                            is_readonly:  prop.readonly,
                            is_private:   matches!(prop.accessibility, Some(Accessibility::Private)),
                            is_protected: matches!(prop.accessibility, Some(Accessibility::Protected)),
                            is_override:  prop.is_override,
                            is_abstract:  prop.is_abstract,
                        };
                        members.push(HirClassMember::Field(field_name, ty, init, mods));
                    }
                }

                // ── Private fields (#x) ──────────────────────────────────────────────
                SwcClassMember::PrivateProp(prop) => {
                    let field_name = format!("private${}", prop.key.id.sym);
                    let ty = self.extract_type(prop.type_ann.as_deref());
                    let init = prop.value.as_ref().map(|v| self.build_expr(v));
                    let mods = FieldModifiers {
                        is_static:    prop.is_static,
                        is_readonly:  false,
                        is_private:   true,
                        is_protected: false,
                        is_override:  false,
                        is_abstract:  false,
                    };
                    members.push(HirClassMember::Field(field_name, ty, init, mods));
                }

                // ── Private methods (#m) ─────────────────────────────────────────────
                SwcClassMember::PrivateMethod(method) => {
                    let method_name = format!("private${}", method.key.id.sym);
                    let mut args = Vec::new();
                    let mut prepended_stmts = Vec::new();
                    for (i, param) in method.function.params.iter().enumerate() {
                        match &param.pat {
                            Pat::Ident(binding) => {
                                let arg_name = binding.id.sym.to_string();
                                let arg_type = self.extract_type(binding.type_ann.as_deref());
                                self.env.bind(arg_name.clone(), arg_type.clone());
                                args.push((arg_name, arg_type));
                            }
                            Pat::Rest(rest) => {
                                if let Pat::Ident(binding) = &*rest.arg {
                                    let arg_name = binding.id.sym.to_string();
                                    let arg_type = self.extract_type(rest.type_ann.as_deref());
                                    self.env.bind(arg_name.clone(), arg_type.clone());
                                    args.push((arg_name, arg_type));
                                }
                            }
                            other_pat => {
                                let synth_name = format!("_param_{}", i);
                                args.push((synth_name.clone(), Type::Any));
                                if let Some(destruct) = self.build_destructuring_stmt(other_pat, HirExpr::Var(synth_name, Type::Any)) {
                                    prepended_stmts.push(destruct);
                                }
                            }
                        }
                    }
                    let ret_type = self.extract_type(method.function.return_type.as_deref());
                    let mut body_stmts = self.build_block_stmt(method.function.body.as_ref().unwrap());
                    body_stmts.splice(0..0, prepended_stmts);
                    let mut mods = MethodModifiers {
                        is_static:    method.is_static,
                        is_abstract:  false,
                        is_private:   true,
                        is_protected: false,
                        is_async:     method.function.is_async,
                        is_override:  false,
                    };
                    if method.function.is_async {
                        mods.is_async = false;
                        let closure = HirExpr::Arrow(vec![], ret_type.clone(), body_stmts, FnModifiers { is_async: false, is_generator: false, is_export: false });
                        let spawn = HirExpr::VirtualThreadSpawn(Box::new(closure));
                        body_stmts = vec![HirStmt::Return(Some(spawn))];
                    }
                    members.push(HirClassMember::Method(method_name, args, ret_type, body_stmts, mods));
                }

                // ── Instance / static methods ────────────────────────────────────────
                SwcClassMember::Method(method) if method.kind == MethodKind::Method => {
                    if let PropName::Ident(ident) = &method.key {
                        let method_name = ident.sym.to_string();
                        let mut args = Vec::new();
                        let mut prepended_stmts = Vec::new();
                        for (i, param) in method.function.params.iter().enumerate() {
                            match &param.pat {
                                Pat::Ident(binding) => {
                                    let arg_name = binding.id.sym.to_string();
                                    let arg_type = self.extract_type(binding.type_ann.as_deref());
                                    self.env.bind(arg_name.clone(), arg_type.clone());
                                    args.push((arg_name, arg_type));
                                }
                                Pat::Rest(rest) => {
                                    if let Pat::Ident(binding) = &*rest.arg {
                                        let arg_name = binding.id.sym.to_string();
                                        let arg_type = self.extract_type(rest.type_ann.as_deref());
                                        self.env.bind(arg_name.clone(), arg_type.clone());
                                        args.push((arg_name, arg_type));
                                    }
                                }
                                other_pat => {
                                    let synth_name = format!("_param_{}", i);
                                    args.push((synth_name.clone(), Type::Any));
                                    if let Some(destruct) = self.build_destructuring_stmt(other_pat, HirExpr::Var(synth_name, Type::Any)) {
                                        prepended_stmts.push(destruct);
                                    }
                                }
                            }
                        }
                        let ret_type = self.extract_type(method.function.return_type.as_deref());
                        let mut body_stmts = self.build_block_stmt(method.function.body.as_ref().unwrap());
                        body_stmts.splice(0..0, prepended_stmts);
                        let mut mods = MethodModifiers {
                            is_static:    method.is_static,
                            is_abstract:  method.function.body.is_none(),
                            is_private:   matches!(method.accessibility, Some(Accessibility::Private)),
                            is_protected: matches!(method.accessibility, Some(Accessibility::Protected)),
                            is_async:     method.function.is_async,
                            is_override:  method.is_override,
                        };
                        if method.function.is_async {
                            mods.is_async = false;
                            let closure = HirExpr::Arrow(vec![], ret_type.clone(), body_stmts, FnModifiers { is_async: false, is_generator: false, is_export: false });
                            let spawn = HirExpr::VirtualThreadSpawn(Box::new(closure));
                            body_stmts = vec![HirStmt::Return(Some(spawn))];
                        }
                        members.push(HirClassMember::Method(method_name, args, ret_type, body_stmts, mods));
                    }
                }

                // ── Constructor ──────────────────────────────────────────────────────
                SwcClassMember::Constructor(ctor) => {
                    let mut args = Vec::new();
                    let mut prepended_stmts = Vec::new();
                    for (i, param) in ctor.params.iter().enumerate() {
                        match param {
                            ParamOrTsParamProp::Param(p) => {
                                match &p.pat {
                                    Pat::Ident(binding) => {
                                        let arg_name = binding.id.sym.to_string();
                                        let arg_type = self.extract_type(binding.type_ann.as_deref());
                                        self.env.bind(arg_name.clone(), arg_type.clone());
                                        args.push((arg_name, arg_type));
                                    }
                                    other_pat => {
                                        let synth_name = format!("_param_{}", i);
                                        args.push((synth_name.clone(), Type::Any));
                                        if let Some(destruct) = self.build_destructuring_stmt(other_pat, HirExpr::Var(synth_name, Type::Any)) {
                                            prepended_stmts.push(destruct);
                                        }
                                    }
                                }
                            }
                            // Parameter properties: `constructor(public x: number)`
                            ParamOrTsParamProp::TsParamProp(ts_prop) => {
                                if let TsParamPropParam::Ident(binding) = &ts_prop.param {
                                    let arg_name = binding.id.sym.to_string();
                                    let arg_type = self.extract_type(binding.type_ann.as_deref());
                                    self.env.bind(arg_name.clone(), arg_type.clone());
                                    args.push((arg_name.clone(), arg_type.clone()));
                                    // Also emit a field for the parameter property
                                    let mods = FieldModifiers {
                                        is_static:    false,
                                        is_readonly:  ts_prop.readonly,
                                        is_private:   matches!(ts_prop.accessibility, Some(Accessibility::Private)),
                                        is_protected: matches!(ts_prop.accessibility, Some(Accessibility::Protected)),
                                        is_override:  ts_prop.is_override,
                                        is_abstract:  false,
                                    };
                                    members.push(HirClassMember::Field(arg_name, arg_type, None, mods));
                                }
                            }
                        }
                    }
                    let mut body_stmts = self.build_block_stmt(ctor.body.as_ref().unwrap());
                    body_stmts.splice(0..0, prepended_stmts);
                    members.push(HirClassMember::Constructor(args, body_stmts));
                }

                // ── Static block ─────────────────────────────────────────────────────
                SwcClassMember::StaticBlock(block) => {
                    let body = self.build_block_stmt(&block.body);
                    members.push(HirClassMember::StaticInit(body));
                }

                _ => {}
            }
        }

        // Re-process methods to properly split getters/setters (SWC puts them in Method with kind)
        // We do a second pass over the raw body specifically for Getter/Setter kinds
        for member in &class_decl.class.body {
            if let SwcClassMember::Method(method) = member {
                match method.kind {
                    MethodKind::Getter => {
                        if let PropName::Ident(ident) = &method.key {
                            let prop_name = ident.sym.to_string();
                            let ret_type = self.extract_type(method.function.return_type.as_deref());
                            let body = self.build_block_stmt(method.function.body.as_ref().unwrap());
                            // Replace the Method we pushed earlier (it will have been added above)
                            // Actually: add as Getter — the emitter will deduplicate by name if needed
                            members.push(HirClassMember::Getter(prop_name, ret_type, body, method.is_static));
                        }
                    }
                    MethodKind::Setter => {
                        if let PropName::Ident(ident) = &method.key {
                            let prop_name = ident.sym.to_string();
                            // Setter has exactly one parameter
                            let param_name = method.function.params.first()
                                .and_then(|p| if let Pat::Ident(id) = &p.pat { Some(id.id.sym.to_string()) } else { None })
                                .unwrap_or_else(|| "value".to_string());
                            let param_type = method.function.params.first()
                                .and_then(|p| if let Pat::Ident(id) = &p.pat { Some(self.extract_type(id.type_ann.as_deref())) } else { None })
                                .unwrap_or(Type::Any);
                            let body = self.build_block_stmt(method.function.body.as_ref().unwrap());
                            members.push(HirClassMember::Setter(prop_name, param_name, param_type, body, method.is_static));
                        }
                    }
                    MethodKind::Method => {} // already handled in first pass
                }
            }
        }

        // Remove Method entries that were actually getters/setters (they got re-added as Getter/Setter)
        // (Keep only those with kind == Method from the first pass — we need to dedup)
        // Strategy: rebuild members filtering out duplicate Method entries for getter/setter names
        let getter_setter_names: std::collections::HashSet<String> = class_decl.class.body.iter()
            .filter_map(|m| {
                if let SwcClassMember::Method(method) = m {
                    if method.kind != MethodKind::Method {
                        if let PropName::Ident(id) = &method.key {
                            return Some(id.sym.to_string());
                        }
                    }
                }
                None
            })
            .collect();

        members.retain(|m| {
            if let HirClassMember::Method(name, _, _, _, _) = m {
                !getter_setter_names.contains(name)
            } else {
                true
            }
        });

        let extends = class_decl.class.super_class.as_ref().map(|sc| {
            if let Expr::Ident(id) = &**sc { id.sym.to_string() } else { "java/lang/Object".to_string() }
        });
        let implements = class_decl.class.implements.iter().filter_map(|imp| {
            if let Expr::Ident(id) = &*imp.expr { Some(id.sym.to_string()) } else { None }
        }).collect();

        let is_abstract = class_decl.class.is_abstract;
        self.current_class = old_class;

        Some(HirStmt::ClassDecl(name, ClassDef {
            extends,
            implements,
            is_abstract,
            members,
        }))
    }
}
