use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_env::TypeEnv;

pub mod expr;
pub mod stmt;
pub mod types;

pub struct IrBuilder {
    pub(crate) env: TypeEnv,
    pub(crate) current_class: Option<String>,
    pub(crate) synthetic_classes: Vec<HirStmt>,
    pub(crate) next_class_expr_id: usize,
    pub(crate) next_temp_id: usize,
}

impl IrBuilder {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            current_class: None,
            synthetic_classes: Vec::new(),
            next_class_expr_id: 1,
            next_temp_id: 1,
        }
    }

    pub(crate) fn next_temp_name(&mut self) -> String {
        let name = format!("$temp${}", self.next_temp_id);
        self.next_temp_id += 1;
        name
    }

    pub fn build_module(&mut self, module: &Module) -> Vec<HirStmt> {
        let mut stmts = Vec::new();
        for item in &module.body {
            if let ModuleItem::Stmt(stmt) = item {
                if let Some(hir_stmt) = self.build_stmt(stmt) {
                    stmts.push(hir_stmt);
                }
            } else if let ModuleItem::ModuleDecl(decl) = item {
                match decl {
                    ModuleDecl::ExportDefaultDecl(export) => {
                        if let DefaultDecl::Fn(fn_decl) = &export.decl {
                            let name = fn_decl.ident.as_ref().map(|i| i.sym.to_string()).unwrap_or_else(|| "App".to_string());
                            let mut args = Vec::new();
                            for param in &fn_decl.function.params {
                                if let Pat::Ident(binding) = &param.pat {
                                    let arg_name = binding.id.sym.to_string();
                                    let arg_type = self.extract_type(binding.type_ann.as_deref());
                                    args.push(FnArg { name: arg_name.clone(), ty: arg_type.clone(), is_rest: false });
                                    self.env.bind(arg_name, arg_type);
                                } else if let Pat::Rest(rest) = &param.pat {
                                    if let Pat::Ident(binding) = &*rest.arg {
                                        let arg_name = binding.id.sym.to_string();
                                        let arg_type = self.extract_type(binding.type_ann.as_deref());
                                        args.push(FnArg { name: arg_name.clone(), ty: arg_type.clone(), is_rest: true });
                                        self.env.bind(arg_name, arg_type);
                                    }
                                }
                            }
                            let ret_type = self.extract_type(fn_decl.function.return_type.as_deref());
                            let body_stmts = self.build_block_stmt(&fn_decl.function.body.as_ref().unwrap());
                            
                            let modifiers = FnModifiers {
                                is_async: fn_decl.function.is_async,
                                is_generator: fn_decl.function.is_generator,
                                is_export: true,
                            };
                            stmts.push(HirStmt::Export(Box::new(HirStmt::FnDecl(name, args, ret_type, body_stmts, modifiers))));
                        }
                    }
                    ModuleDecl::ExportDecl(export) => {
                        if let Some(stmt) = self.build_stmt(&Stmt::Decl(export.decl.clone())) {
                            stmts.push(HirStmt::Export(Box::new(stmt)));
                        }
                    }
                    ModuleDecl::ExportNamed(export) => {
                        if export.type_only {
                            continue;
                        }
                        if let Some(src) = &export.src {
                            let source = src.value.to_string();
                            let mut import_bindings = Vec::new();
                            for specifier in &export.specifiers {
                                match specifier {
                                    ExportSpecifier::Named(named) => {
                                        let orig = match &named.orig {
                                            ModuleExportName::Ident(id) => id.sym.to_string(),
                                            ModuleExportName::Str(s) => s.value.to_string(),
                                        };
                                        let exported_name = named.exported.as_ref().map(|e| match e {
                                            ModuleExportName::Ident(id) => id.sym.to_string(),
                                            ModuleExportName::Str(s) => s.value.to_string(),
                                        }).unwrap_or_else(|| orig.clone());

                                        let temp_local = format!("$reexport${}${}", orig, self.next_temp_id);
                                        self.next_temp_id += 1;

                                        self.env.bind(temp_local.clone(), Type::Any);
                                        import_bindings.push(ImportBinding { name: orig.clone(), alias: Some(temp_local.clone()) });

                                        self.env.bind(exported_name.clone(), Type::Any);
                                        stmts.push(HirStmt::Export(Box::new(HirStmt::Let(
                                            exported_name,
                                            Type::Any,
                                            HirExpr::Var(temp_local, Type::Any)
                                        ))));
                                    }
                                    ExportSpecifier::Namespace(ns) => {
                                        let exported_name = match &ns.name {
                                            ModuleExportName::Ident(id) => id.sym.to_string(),
                                            ModuleExportName::Str(s) => s.value.to_string(),
                                        };

                                        let temp_local = format!("$reexport$ns${}", self.next_temp_id);
                                        self.next_temp_id += 1;

                                        self.env.bind(temp_local.clone(), Type::Any);
                                        import_bindings.push(ImportBinding { name: "*".to_string(), alias: Some(temp_local.clone()) });

                                        self.env.bind(exported_name.clone(), Type::Any);
                                        stmts.push(HirStmt::Export(Box::new(HirStmt::Let(
                                            exported_name,
                                            Type::Any,
                                            HirExpr::Var(temp_local, Type::Any)
                                        ))));
                                    }
                                    _ => {}
                                }
                            }
                            if !import_bindings.is_empty() {
                                stmts.push(HirStmt::Import(import_bindings, source));
                            }
                        }
                    }
                    ModuleDecl::ExportAll(export) => {
                        if export.type_only {
                            continue;
                        }
                        let source = export.src.value.to_string();
                        let temp_ns = format!("$reexport$all${}", self.next_temp_id);
                        self.next_temp_id += 1;
                        self.env.bind(temp_ns.clone(), Type::Any);
                        stmts.push(HirStmt::Import(vec![ImportBinding { name: "*".to_string(), alias: Some(temp_ns) }], source));
                    }
                    ModuleDecl::Import(import) => {
                        if import.type_only {
                            continue;
                        }
                        let source = import.src.value.to_string();
                        let mut bindings = Vec::new();
                        for specifier in &import.specifiers {
                            match specifier {
                                swc_core::ecma::ast::ImportSpecifier::Named(named) => {
                                    if named.is_type_only {
                                        continue;
                                    }
                                    let local = named.local.sym.to_string();
                                    let imported = named.imported.as_ref().map(|i| {
                                        if let swc_core::ecma::ast::ModuleExportName::Ident(id) = i {
                                            id.sym.to_string()
                                        } else {
                                            local.clone()
                                        }
                                    });
                                    bindings.push(ImportBinding { name: imported.unwrap_or_else(|| local.clone()), alias: Some(local) });
                                }
                                swc_core::ecma::ast::ImportSpecifier::Default(def) => {
                                    bindings.push(ImportBinding { name: "default".to_string(), alias: Some(def.local.sym.to_string()) });
                                }
                                swc_core::ecma::ast::ImportSpecifier::Namespace(ns) => {
                                    bindings.push(ImportBinding { name: "*".to_string(), alias: Some(ns.local.sym.to_string()) });
                                }
                            }
                        }
                        stmts.push(HirStmt::Import(bindings, source));
                    }
                    _ => {}
                }
            }
        }
        // Append all dynamically collected synthetic declarations (such as class expressions)
        stmts.extend(self.synthetic_classes.drain(..));
        stmts
    }
}
