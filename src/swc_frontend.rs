use std::sync::Arc;
use swc_core::{
    common::{
        errors::{Handler, Emitter, DiagnosticBuilder},
        FileName, SourceMap,
    },
    ecma::{
        ast::{Module, EsVersion},
        parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig},
    },
};

struct SimpleEmitter;
impl Emitter for SimpleEmitter {
    fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
        println!("{}", db.message());
    }
}

pub fn parse_typescript(source: &str, file_name: &str) -> Module {
    let cm: Arc<SourceMap> = Arc::default();
    let handler = Handler::with_emitter(
        true,
        false,
        Box::new(SimpleEmitter),
    );

    let fm = cm.new_source_file(
        FileName::Custom(file_name.into()),
        source.into(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: true, 
            decorators: true,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        EsVersion::Es2022,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    parser
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("Failed to parse TypeScript/TSX")
}
