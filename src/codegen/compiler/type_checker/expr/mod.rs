pub mod literal;
pub mod var;
pub mod this;
pub mod bin_op;
pub mod unary_op;
pub mod ternary;
pub mod call;
pub mod method_call;
pub mod super_call;
pub mod new;
pub mod field;
pub mod index;
pub mod optional_chain;
pub mod array_lit;
pub mod object_lit;
pub mod delete_prop;
pub mod object_method;
pub mod cast;
pub mod instance_of;
pub mod template_lit;
pub mod arrow;
pub mod spread;
pub mod await_expr;
pub mod update;
pub mod logical;
pub mod array_ops;

use crate::codegen::compiler::ir::*;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_expr(&mut self, expr: &HirExpr) -> Type {
        match expr {
            HirExpr::IntLit(_) | HirExpr::DoubleLit(_) | HirExpr::StringLit(_) | 
            HirExpr::BoolLit(_) | HirExpr::NullLit | HirExpr::UndefinedLit |
            HirExpr::BigIntLit(_) | HirExpr::RegExpLit(_, _) => self.check_literal(expr),
            HirExpr::Var(name, _) => self.check_var(name),
            HirExpr::This(ty) => self.check_this(ty),
            HirExpr::BinOp(op, left, right, _) => self.check_bin_op(op, left, right),
            HirExpr::UnaryOp(op, arg, _) => self.check_unary_op(op, arg),
            HirExpr::Ternary(cond, cons, alt, _) => self.check_ternary(cond, cons, alt),
            HirExpr::Call(name, args, _) => self.check_call(name, args),
            HirExpr::MethodCall(obj, method, args, _) => self.check_method_call(obj, method, args),
            HirExpr::SuperCall(name, args, _) => self.check_super_call(name, args),
            HirExpr::New(class, args, _) => self.check_new(class, args),
            HirExpr::FieldGet(obj, prop_name, _) => self.check_field_get(obj, prop_name),
            HirExpr::FieldSet(obj, prop_name, val, _) => self.check_field_set(obj, prop_name, val),
            HirExpr::IndexGet(obj, idx, _) => self.check_index_get(obj, idx),
            HirExpr::IndexSet(obj, idx, val, _) => self.check_index_set(obj, idx, val),
            HirExpr::OptionalChain(obj, prop, _) => self.check_optional_chain(obj, prop),
            HirExpr::ArrayLit(elems, _) => self.check_array_lit(elems),
            HirExpr::ObjectLit(props, _) => self.check_object_lit(props),
            HirExpr::DeleteProp(obj, prop) => self.check_delete_prop(obj, prop),
            HirExpr::ObjectMethod(method, args, ret_ty) => self.check_object_method(method, args, ret_ty),
            HirExpr::Cast(expr, cast_ty) => self.check_cast(expr, cast_ty),
            HirExpr::InstanceOf(expr, class) => self.check_instance_of(expr, class),
            HirExpr::TemplateLit(parts) => self.check_template_lit(parts),
            HirExpr::Arrow(args, ret_type, body, _) => self.check_arrow(args, ret_type, body),
            HirExpr::Spread(expr) => self.check_spread(expr),
            HirExpr::Await(expr, _) => self.check_await(expr),
            HirExpr::PreIncrement(name) | HirExpr::PostIncrement(name) 
            | HirExpr::PreDecrement(name) | HirExpr::PostDecrement(name) => self.check_update(name),
            HirExpr::LogicalAnd(l, r, _) | HirExpr::LogicalOr(l, r, _) => self.check_logical(l, r),
            HirExpr::ArrayLen(obj) => self.check_array_len(obj),
            HirExpr::ArrayMethod(obj, method, args, ty) => self.check_array_method(obj, method, args, ty),
            HirExpr::OptionalCall(callee, args, _) => self.check_optional_call(callee, args),
            HirExpr::DynamicCall(callee, args, _) => self.check_dynamic_call(callee, args),
            HirExpr::VirtualThreadSpawn(closure) => self.check_expr(closure),
            HirExpr::DynamicImport(path) => {
                self.check_expr(path);
                Type::Class("java.util.concurrent.CompletableFuture".to_string())
            }
            HirExpr::TaggedTemplate(tag, quasis, exprs) => {
                self.check_expr(tag);
                for q in quasis { self.check_expr(q); }
                for e in exprs { self.check_expr(e); }
                Type::Any
            }
            HirExpr::Yield(arg, _) => {
                if let Some(a) = arg {
                    self.check_expr(a);
                }
                Type::Any
            }
            HirExpr::GeneratorSentinel => Type::Any,
            HirExpr::Seq(exprs, _) => {
                let mut last_ty = Type::Any;
                for expr in exprs {
                    last_ty = self.check_expr(expr);
                }
                last_ty
            }
        }
    }
}
