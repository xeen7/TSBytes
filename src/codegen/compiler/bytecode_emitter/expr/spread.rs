use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_spread(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Spread(inner) => {
                            self.emit_expr(inner); // pass through
                        }
            _ => unreachable!(),
        }
    }
}
