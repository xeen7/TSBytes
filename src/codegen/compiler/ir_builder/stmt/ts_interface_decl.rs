use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ts_interface_decl(&mut self, iface: &TsInterfaceDecl) -> Option<HirStmt> {
        let name = iface.id.sym.to_string();
        let extends = iface.extends.iter().filter_map(|e| {
            if let Expr::Ident(id) = &*e.expr { Some(id.sym.to_string()) } else { None }
        }).collect();
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        for member in &iface.body.body {
            if let TsTypeElement::TsMethodSignature(sig) = member {
                if let Expr::Ident(id) = &*sig.key {
                    let method_name = id.sym.to_string();
                    let mut args = Vec::new();
                    for param in &sig.params {
                        if let TsFnParam::Ident(binding) = param {
                            let arg_name = binding.id.sym.to_string();
                            let arg_type = self.extract_type(binding.type_ann.as_deref());
                            args.push(FnArg { name: arg_name, ty: arg_type, is_rest: false });
                        } else if let TsFnParam::Rest(rest) = param {
                            if let Pat::Ident(binding) = &*rest.arg {
                                let arg_name = binding.id.sym.to_string();
                                let arg_type = self.extract_type(binding.type_ann.as_deref());
                                args.push(FnArg { name: arg_name, ty: arg_type, is_rest: true });
                            }
                        }
                    }
                    let ret_type = self.extract_type(sig.type_ann.as_deref());
                    methods.push(InterfaceMethod { name: method_name, args, return_type: ret_type });
                }
            } else if let TsTypeElement::TsPropertySignature(sig) = member {
                if let Expr::Ident(id) = &*sig.key {
                    let field_name = id.sym.to_string();
                    let field_ty = self.extract_type(sig.type_ann.as_deref());
                    fields.push((field_name, field_ty));
                }
            }
        }
        self.env.register_type(name.clone(), Type::Object(fields.clone()));
        Some(HirStmt::InterfaceDecl(name, InterfaceDef { extends, methods, fields }))
    }
}
