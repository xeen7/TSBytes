pub mod swc_frontend;
pub mod codegen;
pub mod cli;
pub mod error;

use std::path::Path;

use error::Result;


/// Compile TSX → JVM bytecode (.class files) to a directory.
pub fn compile(
    source: &str,
    package_name: &str,
    output_dir: &Path,
) -> Result<()> {
    let program = swc_frontend::parse_typescript(source, "input.tsx");
    let gen = codegen::CodeGenerator::new(package_name, "TSDroid App");
    gen.emit_bytecode_to_dir(&program, output_dir)
}

/// Compile TSX → single Executable JAR archive.
pub fn compile_to_jar(
    source: &str,
    package_name: &str,
    output_jar: &Path,
) -> Result<()> {
    let program = swc_frontend::parse_typescript(source, "input.tsx");
    let gen = codegen::CodeGenerator::new(package_name, "TSDroid App");
    gen.build_jar(&program, output_jar)
}

/// Compile TSX → raw .class file bytes (in-memory).
pub fn compile_to_bytes(
    source: &str,
    package_name: &str,
) -> Result<Vec<(String, Vec<u8>)>> {
    let program = swc_frontend::parse_typescript(source, "input.tsx");
    let gen = codegen::CodeGenerator::new(package_name, "TSDroid App");
    gen.emit_bytecode(&program)
}

/// Run type checker on TSX and return any errors found.
pub fn type_check_to_errors(
    source: &str,
) -> Result<Vec<String>> {
    let module = swc_frontend::parse_typescript(source, "input.tsx");
    Ok(codegen::compiler::type_check_program(&module))
}
