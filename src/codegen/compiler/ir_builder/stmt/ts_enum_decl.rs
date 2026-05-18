use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ts_enum_decl(&mut self, enum_decl: &TsEnumDecl) -> Option<HirStmt> {
        let name = enum_decl.id.sym.to_string();
        let mut members = Vec::new();
        let mut next_val = 0.0;
        let mut fields_ty = Vec::new();

        for member in &enum_decl.members {
            let member_name = match &member.id {
                TsEnumMemberId::Ident(id) => id.sym.to_string(),
                TsEnumMemberId::Str(s) => s.value.to_string(),
            };

            let init = if let Some(init_expr) = &member.init {
                let parsed = self.build_expr(init_expr);
                if let HirExpr::IntLit(val) = &parsed {
                    next_val = *val as f64 + 1.0;
                } else if let HirExpr::DoubleLit(val) = &parsed {
                    next_val = *val + 1.0;
                }
                parsed
            } else {
                let parsed = HirExpr::DoubleLit(next_val);
                next_val += 1.0;
                parsed
            };

            let mods = FieldModifiers {
                is_static: true,
                is_readonly: true,
                is_private: false,
                is_protected: false,
                is_override: false,
                is_abstract: false,
            };

            fields_ty.push((member_name.clone(), Type::Double));
            members.push(crate::codegen::compiler::ir::ClassMember::Field(member_name, Type::Double, Some(init), mods));
        }

        // Register the enum name as an Object type of its fields
        self.env.register_type(name.clone(), Type::Object(fields_ty));

        Some(HirStmt::ClassDecl(name, ClassDef {
            extends: None,
            implements: Vec::new(),
            is_abstract: false,
            members,
        }))
    }
}
