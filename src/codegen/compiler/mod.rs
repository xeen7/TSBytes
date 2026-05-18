pub mod ir;
pub mod type_env;
pub mod ir_builder;
pub mod type_checker;
pub mod bytecode_emitter;
pub mod closure_pass;
pub mod runtime_gen;
pub mod module_graph;
pub mod mir;
pub mod hir_to_mir;
pub mod mir_opt;
pub mod runtime_bytes;
use swc_core::ecma::ast::Module;
use ir_builder::IrBuilder;
use type_checker::TypeChecker;
use bytecode_emitter::BytecodeEmitter;
use closure_pass::lower_closures;

/// Compile a single TypeScript module (parsed by SWC) into JVM bytecode.
/// The closure pass runs automatically before bytecode emission.
pub fn compile_program(module: &Module, package_name: &str) -> Vec<(String, Vec<u8>)> {
    let mut builder = IrBuilder::new();
    let hir = builder.build_module(module);

    let mut checker = TypeChecker::new();
    let type_errors = checker.check_program(&hir);
    if !type_errors.is_empty() {
        for err in type_errors {
            eprintln!("{}", err);
        }
    }

    // Run the closure lowering pass: transforms arrow functions
    // into synthetic class declarations with captured variables
    let hir = lower_closures(hir);

    let emitter = BytecodeEmitter::new();
    let mut results = emitter.emit_program(&hir, package_name);

    // Bundle the TsObject runtime class into the output
    let (ts_obj_path, ts_obj_bytes) = runtime_gen::generate_ts_object_class();
    results.push((ts_obj_path, ts_obj_bytes));

    // Bundle the TsError runtime class into the output
    let (ts_err_path, ts_err_bytes) = runtime_gen::generate_ts_error_class();
    results.push((ts_err_path, ts_err_bytes));

    // Bundle the TsRuntime runtime class into the output
    let (ts_run_path, ts_run_bytes) = runtime_gen::generate_ts_runtime_class();
    results.push((ts_run_path, ts_run_bytes));

    // Bundle the TsGenerator runtime class into the output
    let (ts_gen_path, ts_gen_bytes) = runtime_gen::generate_ts_generator_class();
    results.push((ts_gen_path, ts_gen_bytes));

    // Bundle the TsDate runtime class into the output
    let (ts_date_path, ts_date_bytes) = runtime_gen::generate_ts_date_class();
    results.push((ts_date_path, ts_date_bytes));

    // Bundle the TsMap runtime class into the output
    let (ts_map_path, ts_map_bytes) = runtime_gen::generate_ts_map_class();
    results.push((ts_map_path, ts_map_bytes));

    // Bundle the TsSet runtime class into the output
    let (ts_set_path, ts_set_bytes) = runtime_gen::generate_ts_set_class();
    results.push((ts_set_path, ts_set_bytes));

    // Bundle the TsWeakMap runtime class into the output
    let (ts_weakmap_path, ts_weakmap_bytes) = runtime_gen::generate_ts_weakmap_class();
    results.push((ts_weakmap_path, ts_weakmap_bytes));

    // Bundle the TsWeakSet runtime class into the output
    let (ts_weakset_path, ts_weakset_bytes) = runtime_gen::generate_ts_weakset_class();
    results.push((ts_weakset_path, ts_weakset_bytes));

    // Bundle the TsJSON runtime class into the output
    let (ts_json_path, ts_json_bytes) = runtime_gen::generate_ts_json_class();
    results.push((ts_json_path, ts_json_bytes));

    // Bundle the TsJSON$Parser runtime class into the output
    let (ts_json_parser_path, ts_json_parser_bytes) = runtime_gen::generate_ts_json_parser_class();
    results.push((ts_json_parser_path, ts_json_parser_bytes));

    // Bundle the TsRegExp runtime class into the output
    let (ts_regexp_path, ts_regexp_bytes) = runtime_gen::generate_ts_regexp_class();
    results.push((ts_regexp_path, ts_regexp_bytes));

    // Bundle the TsResponse runtime class into the output
    let (ts_resp_path, ts_resp_bytes) = runtime_gen::generate_ts_response_class();
    results.push((ts_resp_path, ts_resp_bytes));

    results
}

/// Run only the type checker and return any type errors
pub fn type_check_program(module: &Module) -> Vec<String> {
    let mut builder = IrBuilder::new();
    let hir = builder.build_module(module);
    let mut checker = TypeChecker::new();
    checker.check_program(&hir)
}
