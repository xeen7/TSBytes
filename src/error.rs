use colored::Colorize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Error, Debug)]
pub enum TsDroidError {
    // Lexer errors
    #[error("Unexpected character '{ch}'")]
    UnexpectedCharacter { ch: char, span: Span },

    #[error("Unterminated string literal")]
    UnterminatedString { span: Span },

    #[error("Unterminated block comment")]
    UnterminatedComment { span: Span },

    // Parser errors
    #[error("Unexpected token: expected {expected}, got {got:?}")]
    UnexpectedToken {
        expected: String,
        got: String,
        span: Span,
    },

    #[error("Unclosed JSX tag '<{tag}>'")]
    UnclosedJsxTag { tag: String, span: Span },

    #[error("Mismatched JSX closing tag: expected '</{expected}>', got '</{got}>'")]
    MismatchedJsxTag {
        expected: String,
        got: String,
        span: Span,
    },

    // Analyzer errors
    #[error("Unknown component '<{name}>'")]
    UnknownComponent { name: String, span: Span },

    #[error("Invalid style property '{prop}' for component '<{component}>'")]
    InvalidStyleProp {
        prop: String,
        component: String,
        span: Span,
    },

    // Codegen errors
    #[error("No entry point found. Define a function named 'App'")]
    NoEntryPoint,

    #[error("Codegen error: {0}")]
    Codegen(String),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JAR packaging error: {0}")]
    ZipError(String),
}

impl TsDroidError {
    /// Get the span associated with this error, if any.
    pub fn span(&self) -> Option<&Span> {
        match self {
            Self::UnexpectedCharacter { span, .. } => Some(span),
            Self::UnterminatedString { span } => Some(span),
            Self::UnterminatedComment { span } => Some(span),
            Self::UnexpectedToken { span, .. } => Some(span),
            Self::UnclosedJsxTag { span, .. } => Some(span),
            Self::MismatchedJsxTag { span, .. } => Some(span),
            Self::UnknownComponent { span, .. } => Some(span),
            Self::InvalidStyleProp { span, .. } => Some(span),
            Self::NoEntryPoint => None,
            Self::Codegen(_) => None,
            Self::ZipError(_) => None,
            Self::Io(_) => None,
        }
    }

    /// Pretty-print the error with source context.
    pub fn pretty_print(&self, source: &str, filename: &str) {
        let error_type = match self {
            Self::UnexpectedCharacter { .. } | Self::UnterminatedString { .. } | Self::UnterminatedComment { .. } => {
                "syntax error"
            }
            Self::UnexpectedToken { .. } | Self::UnclosedJsxTag { .. } | Self::MismatchedJsxTag { .. } => {
                "parse error"
            }
            Self::UnknownComponent { .. } | Self::InvalidStyleProp { .. } => "semantic error",
            Self::NoEntryPoint | Self::Codegen(_) => "compile error",
            Self::ZipError(_) => "packaging error",
            Self::Io(_) => "io error",
        };

        eprintln!(
            "{}{} {}",
            "error".red().bold(),
            format!("[{}]", error_type).dimmed(),
            format!("{}", self).white().bold()
        );

        if let Some(span) = self.span() {
            let lines: Vec<&str> = source.lines().collect();
            let line_idx = span.line.saturating_sub(1) as usize;

            if line_idx < lines.len() {
                let line_content = lines[line_idx];
                let line_num = span.line;
                let col = span.col;

                eprintln!(
                    "  {} {}:{}:{}",
                    "-->".blue().bold(),
                    filename,
                    line_num,
                    col
                );
                eprintln!("   {}", "|".blue().bold());
                eprintln!(
                    " {} {} {}",
                    format!("{:>3}", line_num).blue().bold(),
                    "|".blue().bold(),
                    line_content
                );

                let pointer_offset = col.saturating_sub(1) as usize;
                let pointer = format!(
                    "   {} {}{}",
                    "|".blue().bold(),
                    " ".repeat(pointer_offset),
                    "^".red().bold()
                );
                eprintln!("{}", pointer);
            }
        }

        eprintln!();
    }
}

pub type Result<T> = std::result::Result<T, TsDroidError>;
