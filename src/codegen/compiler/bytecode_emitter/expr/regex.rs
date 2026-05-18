use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_regex(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::RegExpLit(pattern, flags) => {
                let pattern_class_idx = self.emitter.cp.add_class("java/util/regex/Pattern").unwrap();

                // Load pattern string
                let pattern_str_idx = self.emitter.cp.add_string(pattern).unwrap();
                self.code.push(Instruction::Ldc_w(pattern_str_idx));

                // Compute flags mask
                let mut flags_int = 0;
                for c in flags.chars() {
                    match c {
                        'i' => flags_int |= 0x02, // CASE_INSENSITIVE
                        'm' => flags_int |= 0x08, // MULTILINE
                        's' => flags_int |= 0x20, // DOTALL
                        'u' => flags_int |= 0x40, // UNICODE_CASE
                        'x' => flags_int |= 0x04, // COMMENTS
                        _ => {}
                    }
                }

                // Load flags integer
                let flags_idx = self.emitter.cp.add_integer(flags_int).unwrap();
                self.code.push(Instruction::Ldc_w(flags_idx));

                // Call Pattern.compile(String, int)
                let compile_method_idx = self.emitter.cp.add_method_ref(
                    pattern_class_idx,
                    "compile",
                    "(Ljava/lang/String;I)Ljava/util/regex/Pattern;"
                ).unwrap();
                self.code.push(Instruction::Invokestatic(compile_method_idx));
            }
            _ => unreachable!(),
        }
    }
}
