use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir::ClassMember as HirClassMember;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ts_module_decl(&mut self, decl: &TsModuleDecl) -> Option<HirStmt> {
        if decl.declare {
            // Ambient namespace / module. Just return None (erase it cleanly).
            return None;
        }

        let name = match &decl.id {
            TsModuleName::Ident(id) => id.sym.to_string(),
            TsModuleName::Str(s) => s.value.to_string(),
        };

        // If there's no body, it is a declare-like ambient definition. Return None.
        let body = decl.body.as_ref()?;

        // A namespace is compiled to a static class on the JVM with static members.
        // We will process the namespace body and convert its exported elements into static fields/methods of the Class definition.
        let mut members = Vec::new();

        if let TsNamespaceBody::TsModuleBlock(block) = body {
            for item in &block.body {
                match item {
                    ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
                        match &export_decl.decl {
                            Decl::Var(var_decl) => {
                                for d in &var_decl.decls {
                                    if let Pat::Ident(ident) = &d.name {
                                        let field_name = ident.id.sym.to_string();
                                        let ty = self.extract_type(ident.type_ann.as_deref());
                                        let init = d.init.as_ref().map(|v| self.build_expr(v));
                                        
                                        // Bind it to the global namespace environment so it can be resolved as Namespace.field
                                        let namespace_qualified_name = format!("{}.{}", name, field_name);
                                        self.env.bind(namespace_qualified_name, ty.clone());
                                        
                                        members.push(HirClassMember::Field(
                                            field_name,
                                            ty,
                                            init,
                                            FieldModifiers {
                                                is_static: true,
                                                is_readonly: var_decl.kind == VarDeclKind::Const,
                                                is_private: false,
                                                is_protected: false,
                                                is_override: false,
                                                is_abstract: false,
                                            },
                                        ));
                                    }
                                }
                            }
                            Decl::Fn(fn_decl) => {
                                let fn_name = fn_decl.ident.sym.to_string();
                                let mut args = Vec::new();
                                for (i, param) in fn_decl.function.params.iter().enumerate() {
                                    if let Pat::Ident(binding) = &param.pat {
                                        args.push((
                                            binding.id.sym.to_string(),
                                            self.extract_type(binding.type_ann.as_deref()),
                                        ));
                                    } else {
                                        args.push((
                                            format!("_param_{}", i),
                                            Type::Any,
                                        ));
                                    }
                                }
                                let ret_type = self.extract_type(fn_decl.function.return_type.as_deref());
                                let body_stmts = self.build_block_stmt(&fn_decl.function.body.as_ref().unwrap());
                                
                                // Bind it to global namespace env
                                let namespace_qualified_name = format!("{}.{}", name, fn_name);
                                let fn_type = Type::Function(
                                    args.iter().map(|a| a.1.clone()).collect(),
                                    Box::new(ret_type.clone()),
                                );
                                self.env.bind(namespace_qualified_name, fn_type);

                                members.push(HirClassMember::Method(
                                    fn_name,
                                    args,
                                    ret_type,
                                    body_stmts,
                                    MethodModifiers {
                                        is_static: true,
                                        is_private: false,
                                        is_protected: false,
                                        is_override: false,
                                        is_abstract: false,
                                        is_async: false,
                                    },
                                ));
                            }
                            _ => {}
                        }
                    }
                    ModuleItem::Stmt(stmt) => {
                        // Standard non-exported statements in a namespace.
                        // We can compile them as a static initializer block or private class members!
                        // For simplicity, if it's a variable or function, compile it as private static member.
                        if let Stmt::Decl(Decl::Var(var_decl)) = stmt {
                            for d in &var_decl.decls {
                                if let Pat::Ident(ident) = &d.name {
                                    let field_name = ident.id.sym.to_string();
                                    let ty = self.extract_type(ident.type_ann.as_deref());
                                    let init = d.init.as_ref().map(|v| self.build_expr(v));
                                    
                                    members.push(HirClassMember::Field(
                                        field_name,
                                        ty,
                                        init,
                                        FieldModifiers {
                                            is_static: true,
                                            is_readonly: var_decl.kind == VarDeclKind::Const,
                                            is_private: true,
                                            is_protected: false,
                                            is_override: false,
                                            is_abstract: false,
                                        },
                                    ));
                                }
                            }
                        } else if let Stmt::Decl(Decl::Fn(fn_decl)) = stmt {
                            let fn_name = fn_decl.ident.sym.to_string();
                            let mut args = Vec::new();
                            for (i, param) in fn_decl.function.params.iter().enumerate() {
                                if let Pat::Ident(binding) = &param.pat {
                                    args.push((
                                        binding.id.sym.to_string(),
                                        self.extract_type(binding.type_ann.as_deref()),
                                    ));
                                } else {
                                    args.push((
                                        format!("_param_{}", i),
                                        Type::Any,
                                    ));
                                }
                            }
                            let ret_type = self.extract_type(fn_decl.function.return_type.as_deref());
                            let body_stmts = self.build_block_stmt(&fn_decl.function.body.as_ref().unwrap());

                            members.push(HirClassMember::Method(
                                fn_name,
                                args,
                                ret_type,
                                body_stmts,
                                MethodModifiers {
                                    is_static: true,
                                    is_private: true,
                                    is_protected: false,
                                    is_override: false,
                                    is_abstract: false,
                                    is_async: false,
                                },
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Register class type in environment
        self.env.bind(name.clone(), Type::Class(name.clone()));

        // Emit the generated static class definition!
        Some(HirStmt::ClassDecl(
            name,
            ClassDef {
                extends: None,
                implements: Vec::new(),
                is_abstract: false,
                members,
            },
        ))
    }
}
