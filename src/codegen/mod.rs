pub mod compiler;
pub mod jar;

use std::path::Path;

use crate::error::Result;
use swc_core::ecma::ast::Module;
use std::fs;


/// Orchestrates the code generation pipeline.
pub struct CodeGenerator {
    package_name: String,
}

impl CodeGenerator {
    pub fn new(package_name: &str, _app_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
        }
    }

    /// Generate JVM .class file bytes directly from the AST.
    pub fn emit_bytecode(&self, program: &Module) -> Result<Vec<(String, Vec<u8>)>> {
        let classes = compiler::compile_program(program, &self.package_name);
        if classes.is_empty() {
            Err(crate::error::TsDroidError::Codegen("No classes generated".into()))
        } else {
            Ok(classes)
        }
    }

    /// Generate JVM .class files to a directory.
    pub fn emit_bytecode_to_dir(&self, program: &Module, output_dir: &Path) -> Result<()> {
        let classes = self.emit_bytecode(program)?;
        for (class_name, bytes) in classes {
            let class_file = output_dir.join(format!("{}.class", class_name));
            if let Some(parent) = class_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(class_file, bytes)?;
        }
        Ok(())
    }



    /// Compile TSX → all classes → packaged into a runnable .jar archive.
    pub fn build_jar(&self, program: &Module, output_jar: &Path) -> Result<()> {
        let classes = compiler::compile_program(program, &self.package_name);
        let main_class_name = format!("{}.App", self.package_name);
        jar::JarPackager::package_jar(&classes, &main_class_name, output_jar)
    }
}
