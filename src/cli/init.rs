//! Project scaffolding — `tsdroid init` command.
//! Creates a standard TypeScript project targeting the JVM via TSDroid.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::Result;

/// Scaffold a new TSDroid project — standard TypeScript console application running on the JVM.
pub fn init_project(dir: &Path, name: &str) -> Result<()> {
    fs::create_dir_all(dir)?;

    let package_name = name.to_lowercase().replace(' ', "-");

    // package.json — standard JS project
    write_package_json(dir, &package_name)?;

    // tsconfig.json — TypeScript config for IDE + type checking
    write_tsconfig(dir)?;

    // Create src directory
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // src/main.ts — starter entrypoint
    write_main_ts(&src_dir)?;

    // .gitignore
    write_gitignore(dir)?;

    // Install dependencies
    install_deps(dir)?;

    Ok(())
}

fn write_package_json(dir: &Path, name: &str) -> Result<()> {
    let content = format!(r#"{{
  "name": "{name}",
  "version": "1.0.0",
  "main": "src/main.ts",
  "scripts": {{
    "build": "tsb jar src/main.ts --output app.jar",
    "typecheck": "tsc --noEmit"
  }},
  "devDependencies": {{
    "typescript": "^5.6.3"
  }}
}}"#);
    fs::write(dir.join("package.json"), content)?;
    Ok(())
}

fn write_tsconfig(dir: &Path) -> Result<()> {
    let content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "noEmit": true
  },
  "include": [
    "src/**/*.ts"
  ]
}"#;
    fs::write(dir.join("tsconfig.json"), content)?;
    Ok(())
}

fn write_main_ts(src_dir: &Path) -> Result<()> {
    let content = r#"function greet(name: string) {
    console.log("Hello, " + name + "!");
    console.log("Welcome to TSBytes - TypeScript running natively on the JVM!");
}

async function main() {
    greet("Developer");

    // Standard library showcase
    let map = new Map<string, number>();
    map.set("TypeScript", 2026);
    map.set("JVM", 8);

    console.log("Map contents:");
    for (const [key, value] of map) {
        console.log("  " + key + " -> " + value);
    }
}

main();
"#;
    fs::write(src_dir.join("main.ts"), content)?;
    Ok(())
}

fn write_gitignore(dir: &Path) -> Result<()> {
    let content = r#"node_modules/
build/
app.jar
.tsbytes_build/
.tsbytes_classes/
"#;
    fs::write(dir.join(".gitignore"), content)?;
    Ok(())
}

fn install_deps(dir: &Path) -> Result<()> {
    let pm = detect_package_manager();
    eprintln!("  ▸ Installing dependencies with {}...", pm);

    let status = Command::new(&pm)
        .arg("install")
        .current_dir(dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            eprintln!("  ▸ Dependencies installed.");
        }
        _ => {
            eprintln!("  ⚠ Could not run '{}'. Run '{} install' manually.", pm, pm);
        }
    }

    Ok(())
}

fn detect_package_manager() -> String {
    for pm in &["bun", "npm", "yarn", "pnpm"] {
        if let Ok(output) = Command::new(pm).arg("--version").output() {
            if output.status.success() {
                return pm.to_string();
            }
        }
    }
    "npm".to_string()
}
