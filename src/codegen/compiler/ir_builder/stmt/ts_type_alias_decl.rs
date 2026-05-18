use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::ir_builder::IrBuilder;

impl IrBuilder {
    pub(crate) fn build_ts_type_alias_decl(&mut self, alias: &TsTypeAliasDecl) -> Option<HirStmt> {
        let name = alias.id.sym.to_string();
        let ty = self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: alias.type_ann.clone() }));
        self.env.register_type(name, ty);
        None
    }
}
