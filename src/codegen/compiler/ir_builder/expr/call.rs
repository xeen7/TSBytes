use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_call_expr(&mut self, call: &CallExpr) -> HirExpr {
        if let Callee::Super(_) = &call.callee {
            let mut args = Vec::new();
            for arg in &call.args {
                let mut expr = self.build_expr(&arg.expr);
                if arg.spread.is_some() {
                    expr = HirExpr::Spread(Box::new(expr));
                }
                args.push(expr);
            }
            return HirExpr::SuperCall("<init>".to_string(), args, Type::Void);
        }
        if let Callee::Import(_) = &call.callee {
            let path_expr = if let Some(arg) = call.args.first() {
                self.build_expr(&arg.expr)
            } else {
                HirExpr::StringLit("".to_string())
            };
            return HirExpr::DynamicImport(Box::new(path_expr));
        }
        if let Callee::Expr(callee) = &call.callee {
            if let Expr::Ident(ident) = &**callee {
                let name = ident.sym.to_string();
                let mut args = Vec::new();
                for arg in &call.args {
                    let mut expr = self.build_expr(&arg.expr);
                    if arg.spread.is_some() {
                        expr = HirExpr::Spread(Box::new(expr));
                    }
                    args.push(expr);
                }
                let ret_type = match self.env.get_type(&name) {
                    Type::Function(_, ret) => *ret,
                    _ => Type::Any,
                };
                return HirExpr::Call(name, args, ret_type);
            } else if let Expr::Member(member) = &**callee {
                if let Expr::Ident(obj_ident) = &*member.obj {
                    if obj_ident.sym.as_ref() == "Object" {
                        if let MemberProp::Ident(method_ident) = &member.prop {
                            let method_name = method_ident.sym.to_string();
                            let mut args = Vec::new();
                            for arg in &call.args {
                                args.push(self.build_expr(&arg.expr));
                            }
                            let ret_type = match method_name.as_str() {
                                "keys" | "values" | "entries" => Type::Array(Box::new(Type::Any)),
                                "assign" | "create" | "freeze" | "seal" | "defineProperty" => Type::Any,
                                "getPrototypeOf" => Type::Any,
                                "hasOwn" => Type::Bool,
                                _ => Type::Any,
                            };
                            return HirExpr::ObjectMethod(method_name, args, ret_type);
                        }
                    }
                }
                let obj = self.build_expr(&member.obj);
                if let MemberProp::Ident(ident) = &member.prop {
                    let method_name = ident.sym.to_string();
                    let mut args = Vec::new();
                    for arg in &call.args {
                        let mut expr = self.build_expr(&arg.expr);
                        if arg.spread.is_some() {
                            expr = HirExpr::Spread(Box::new(expr));
                        }
                        args.push(expr);
                    }
                    if matches!(obj.get_type(), Type::Array(_)) {
                        let ret_type = match method_name.as_str() {
                            "push" | "unshift" => Type::Double,
                            "pop" | "shift" | "find" => Type::Any,
                            "indexOf" | "lastIndexOf" | "findIndex" => Type::Double,
                            "includes" | "some" | "every" => Type::Bool,
                            "join" => Type::StringTy,
                            "slice" | "concat" | "filter" | "map" | "flat" | "reverse" | "sort" | "splice" => {
                                obj.get_type()
                            }
                            "forEach" => Type::Void,
                            "reduce" => Type::Any,
                            "fill" => obj.get_type(),
                            _ => Type::Any,
                        };
                        return HirExpr::ArrayMethod(Box::new(obj), method_name, args, ret_type);
                    }
                    if let HirExpr::Var(ref var_name, _) = obj {
                        if var_name == "super" {
                            return HirExpr::SuperCall(method_name, args, Type::Any);
                        }
                    }
                    return HirExpr::MethodCall(Box::new(obj), method_name, args, Type::Any);
                }
            } else if let Expr::SuperProp(super_prop) = &**callee {
                if let SuperProp::Ident(ident) = &super_prop.prop {
                    let method_name = ident.sym.to_string();
                    let mut args = Vec::new();
                    for arg in &call.args {
                        let mut expr = self.build_expr(&arg.expr);
                        if arg.spread.is_some() {
                            expr = HirExpr::Spread(Box::new(expr));
                        }
                        args.push(expr);
                    }
                    return HirExpr::SuperCall(method_name, args, Type::Any);
                }
            }

            let callee_expr = self.build_expr(callee);
            let mut args = Vec::new();
            for arg in &call.args {
                let mut expr = self.build_expr(&arg.expr);
                if arg.spread.is_some() {
                    expr = HirExpr::Spread(Box::new(expr));
                }
                args.push(expr);
            }
            return HirExpr::DynamicCall(Box::new(callee_expr), args, Type::Any);
        }
        HirExpr::UndefinedLit
    }
}
