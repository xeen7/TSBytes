use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_class_expr(&mut self, class_expr: &ClassExpr) -> HirExpr {
        let class_name = if let Some(ident) = &class_expr.ident {
            ident.sym.to_string()
        } else {
            let name = format!("ClassExpr${}", self.next_class_expr_id);
            self.next_class_expr_id += 1;
            name
        };

        // Construct class declaration from the class expression
        let decl = ClassDecl {
            ident: Ident::new(class_name.clone().into(), class_expr.class.span),
            declare: false,
            class: class_expr.class.clone(),
        };

        // Build the class declaration using the existing build_class_decl
        if let Some(hir_decl) = self.build_class_decl(&decl) {
            self.synthetic_classes.push(hir_decl);
        }

        // Return a reference to the synthetic class constructor
        HirExpr::Var(class_name.clone(), Type::Class(class_name))
    }
}
