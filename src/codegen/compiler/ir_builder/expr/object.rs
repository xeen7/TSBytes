use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_object_lit_expr(&mut self, obj: &ObjectLit) -> HirExpr {
        let mut props = Vec::new();
        for prop_or_spread in &obj.props {
            match prop_or_spread {
                PropOrSpread::Prop(prop) => {
                    match &**prop {
                        Prop::KeyValue(kv) => {
                            let key = match &kv.key {
                                PropName::Ident(id) => id.sym.to_string(),
                                PropName::Str(s) => s.value.to_string(),
                                PropName::Computed(c) => {
                                    let _key_expr = self.build_expr(&c.expr);
                                    "[computed]".to_string()
                                }
                                _ => "unknown".to_string(),
                            };
                            let mut val = self.build_expr(&kv.value);
                            if let HirExpr::Arrow(_, _, _, ref mut mods) = val {
                                mods.is_export = false;
                            }
                            props.push(ObjectProp::KeyValue(key, val));
                        }
                        Prop::Method(method) => {
                            if let PropName::Ident(ident) = &method.key {
                                let method_name = ident.sym.to_string();
                                let mut args = Vec::new();
                                for param in &method.function.params {
                                    if let Pat::Ident(binding) = &param.pat {
                                        let arg_name = binding.id.sym.to_string();
                                        let arg_type = self.extract_type(binding.type_ann.as_deref());
                                        args.push(FnArg { name: arg_name, ty: arg_type, is_rest: false });
                                    }
                                }
                                let ret_type = self.extract_type(method.function.return_type.as_deref());
                                let body = self.build_block_stmt(method.function.body.as_ref().unwrap());
                                props.push(ObjectProp::Method(method_name, args, ret_type, body));
                            }
                        }
                        Prop::Getter(getter) => {
                            if let PropName::Ident(ident) = &getter.key {
                                let name = ident.sym.to_string();
                                let body = self.build_block_stmt(getter.body.as_ref().unwrap());
                                let arrow = HirExpr::Arrow(
                                    vec![FnArg { name: "$this".to_string(), ty: Type::Any, is_rest: false }],
                                    Type::Any,
                                    body,
                                    crate::codegen::compiler::ir::FnModifiers::default()
                                );
                                props.push(ObjectProp::Getter(name, arrow));
                            }
                        }
                        Prop::Setter(setter) => {
                            if let PropName::Ident(ident) = &setter.key {
                                let name = ident.sym.to_string();
                                let param_name = if let Pat::Ident(p) = &*setter.param {
                                    p.id.sym.to_string()
                                } else { "_".to_string() };
                                let body = self.build_block_stmt(setter.body.as_ref().unwrap());
                                let arrow = HirExpr::Arrow(
                                    vec![
                                        FnArg { name: "$this".to_string(), ty: Type::Any, is_rest: false },
                                        FnArg { name: param_name, ty: Type::Any, is_rest: false }
                                    ],
                                    Type::Any,
                                    body,
                                    crate::codegen::compiler::ir::FnModifiers::default()
                                );
                                props.push(ObjectProp::Setter(name, arrow));
                            }
                        }
                        _ => {}
                    }
                }
                PropOrSpread::Spread(spread) => {
                    let expr = self.build_expr(&spread.expr);
                    props.push(ObjectProp::Spread(expr));
                }
            }
        }
        HirExpr::ObjectLit(props, Type::Any)
    }
}
