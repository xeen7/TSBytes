pub mod literal;
pub mod var;
pub mod bin_op;
pub mod unary_op;
pub mod call;
pub mod method_call;
pub mod new;
pub mod field;
pub mod array_lit;
pub mod object_lit;
pub mod cast;
pub mod this_expr;
pub mod ternary;
pub mod super_call;
pub mod index;
pub mod optional_chain;
pub mod instance_of;
pub mod delete_prop;
pub mod object_method;
pub mod template_lit;
pub mod arrow;
pub mod spread;
pub mod await_expr;
pub mod update;
pub mod logical;
pub mod array_ops;
pub mod virtual_thread;
pub mod bigint;
pub mod regex;
pub mod dynamic_import;
pub mod tagged_tpl;
pub mod yield_expr;
pub mod sentinel;


use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use super::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::DoubleLit(..) | HirExpr::IntLit(..) | HirExpr::StringLit(..) | HirExpr::BoolLit(..) | HirExpr::NullLit | HirExpr::UndefinedLit => self.emit_literal(expr),
            HirExpr::Var(..) => self.emit_var(expr),
            HirExpr::BinOp(..) => self.emit_bin_op(expr),
            HirExpr::UnaryOp(..) => self.emit_unary_op(expr),
            HirExpr::Call(..) => self.emit_call(expr),
            HirExpr::MethodCall(..) => self.emit_method_call(expr),
            HirExpr::New(..) => self.emit_new(expr),
            HirExpr::FieldGet(..) | HirExpr::FieldSet(..) => self.emit_field(expr),
            HirExpr::ArrayLit(..) => self.emit_array_lit(expr),
            HirExpr::ObjectLit(..) => self.emit_object_lit(expr),
            HirExpr::Cast(..) => self.emit_cast(expr),
            HirExpr::This(..) => self.emit_this_expr(expr),
            HirExpr::Ternary(..) => self.emit_ternary(expr),
            HirExpr::SuperCall(..) => self.emit_super_call(expr),
            HirExpr::IndexGet(..) | HirExpr::IndexSet(..) => self.emit_index(expr),
            HirExpr::OptionalChain(..) => self.emit_optional_chain(expr),
            HirExpr::OptionalCall(..) => self.emit_optional_call(expr),
            HirExpr::DynamicCall(..) => self.emit_dynamic_call(expr),
            HirExpr::InstanceOf(..) => self.emit_instance_of(expr),
            HirExpr::DeleteProp(..) => self.emit_delete_prop(expr),
            HirExpr::ObjectMethod(..) => self.emit_object_method(expr),
            HirExpr::TemplateLit(..) => self.emit_template_lit(expr),
            HirExpr::Arrow(..) => self.emit_arrow(expr),
            HirExpr::Spread(..) => self.emit_spread(expr),
            HirExpr::Await(..) => self.emit_await_expr(expr),
            HirExpr::PreIncrement(..) | HirExpr::PostIncrement(..) | HirExpr::PreDecrement(..) | HirExpr::PostDecrement(..) => self.emit_update(expr),
            HirExpr::LogicalAnd(..) | HirExpr::LogicalOr(..) => self.emit_logical(expr),
            HirExpr::ArrayLen(..) | HirExpr::ArrayMethod(..) => self.emit_array_ops(expr),
            HirExpr::VirtualThreadSpawn(..) => self.emit_virtual_thread_spawn(expr),
            HirExpr::BigIntLit(..) => self.emit_bigint(expr),
            HirExpr::RegExpLit(..) => self.emit_regex(expr),
            HirExpr::DynamicImport(..) => self.emit_dynamic_import(expr),
            HirExpr::TaggedTemplate(..) => self.emit_tagged_template(expr),
            HirExpr::Yield(..) => self.emit_yield(expr),
            HirExpr::GeneratorSentinel => self.emit_sentinel(expr),
            HirExpr::Seq(exprs, _) => {
                if exprs.is_empty() {
                    self.code.push(Instruction::Aconst_null);
                } else {
                    for (i, expr) in exprs.iter().enumerate() {
                        self.emit_expr(expr);
                        if i < exprs.len() - 1 {
                            let ty = expr.get_type();
                            if ty != Type::Void {
                                match ty {
                                    Type::Double => self.code.push(Instruction::Pop2),
                                    _ => self.code.push(Instruction::Pop),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
