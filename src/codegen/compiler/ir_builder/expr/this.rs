use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_this_expr(&mut self, _this: &ThisExpr) -> HirExpr {
        if let Some(class_name) = &self.current_class {
            HirExpr::This(Type::Class(class_name.clone()))
        } else {
            HirExpr::This(Type::Any)
        }
    }
}
