use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "tsb",
    version,
    about = "Compile TypeScript directly to JVM bytecode and executable JARs",
    long_about = "tsbytes compiles TypeScript source files directly to JVM bytecode, \
then packages them into standard, executable JAR files.\n\
No Kotlin. No Gradle. No runtime. Pure native.\n\n\
Use 'tsb init' to create a new TypeScript-to-JVM project."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new tsbytes project
    Init {
        /// Project directory (created if it doesn't exist)
        #[arg(default_value = ".")]
        dir: PathBuf,

        /// App name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Emit only JVM .class files
    Compile {
        /// Path to the .ts/.tsx source file
        file: PathBuf,

        /// Output directory for .class files
        #[arg(short, long, default_value = "./build")]
        output: PathBuf,

        /// JVM package name
        #[arg(short, long, default_value = "com.tsbytes.app")]
        package: String,
    },

    /// Compile TypeScript to an executable JAR archive
    Jar {
        /// Path to the .ts/.tsx source file
        file: PathBuf,

        /// Output .jar path
        #[arg(short, long, default_value = "./output.jar")]
        output: PathBuf,

        /// JVM package name (e.g. com.example)
        #[arg(short, long, default_value = "com.tsbytes.app")]
        package: String,
    },
}

fn main() {
    let cli = Cli::parse();


    match cli.command {
        Commands::Init { dir, name } => {
            let project_name = name.unwrap_or_else(|| {
                dir.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("my-app")
                    .to_string()
            });

            eprintln!(
                "{} Creating new tsbytes project: {}",
                "⚡".bold(),
                project_name.bold()
            );

            match tsbytes::cli::init::init_project(&dir, &project_name) {
                Ok(()) => {
                    eprintln!("\n  {} Project created!", "✓".green().bold());
                    eprintln!("  {} cd {}", "→".blue().bold(), dir.display());
                    eprintln!("  {} tsb jar src/main.ts", "→".blue().bold());
                }
                Err(e) => {
                    eprintln!("{} {}", "error:".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        Commands::Compile {
            file,
            output,
            package,
        } => {
            let source = read_source(&file);

            eprintln!(
                "{} {} → JVM bytecode",
                "⚡".bold(),
                file.display()
            );

            match tsbytes::compile(&source, &package, &output) {
                Ok(()) => {
                    eprintln!(
                        "  {} .class written to {}/{}",
                        "✓".green().bold(),
                        output.display(),
                        package.replace('.', "/")
                    );
                }
                Err(e) => {
                    e.pretty_print(&source, &file.to_string_lossy());
                    process::exit(1);
                }
            }
        }

        Commands::Jar { file, output, package } => {
            let source = read_source(&file);

            eprintln!(
                "{} {} → JAR ({})",
                "⚡".bold(),
                file.display(),
                output.display()
            );

            match tsbytes::compile_to_jar(&source, &package, &output) {
                Ok(()) => {
                    eprintln!(
                        "  {} JAR written to {}",
                        "✓".green().bold(),
                        output.display()
                    );
                    eprintln!(
                        "  {} Run with: java -jar {}",
                        "→".blue().bold(),
                        output.display()
                    );
                }
                Err(e) => {
                    e.pretty_print(&source, &file.to_string_lossy());
                    process::exit(1);
                }
            }
        }
    }
}

fn read_source(file: &PathBuf) -> String {
    match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{} Could not read '{}': {}",
                "error:".red().bold(),
                file.display(),
                e
            );
            process::exit(1);
        }
    }
}
