/// Module Resolution and Multi-File Compilation
///
/// This module implements the ModuleGraph — a dependency tracker that
/// resolves `import` / `export` statements across multiple TypeScript files
/// and maps them to JVM class references.
///
/// Each .ts file compiles to its own JVM class. Exported functions become
/// public static methods; exported classes become separate .class files.
///
/// Example:
///   services/math.ts  →  com/tsdroid/services/Math.class
///   index.ts          →  com/tsdroid/app/MainActivity.class

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a resolved module in the compilation graph.
#[derive(Clone, Debug)]
pub struct ResolvedModule {
    /// The file path of the source module
    pub source_path: PathBuf,
    /// The JVM class name (e.g. "com/tsdroid/services/Math")
    pub jvm_class_name: String,
    /// Exported symbols: name → type signature
    pub exports: HashMap<String, ExportedSymbol>,
    /// Import references: local alias → (source module path, original name)
    pub imports: Vec<ImportRef>,
}

#[derive(Clone, Debug)]
pub struct ExportedSymbol {
    pub name: String,
    pub kind: ExportKind,
    /// JVM method descriptor (e.g. "(DD)D")
    pub descriptor: String,
}

#[derive(Clone, Debug)]
pub enum ExportKind {
    Function,
    Class,
    Constant,
}

#[derive(Clone, Debug)]
pub struct ImportRef {
    /// Local name used in this module
    pub local_name: String,
    /// The module specifier (e.g. "./services/math")
    pub source_module: String,
    /// The original exported name
    pub original_name: String,
}

/// The ModuleGraph tracks all modules and their inter-dependencies.
pub struct ModuleGraph {
    /// Package prefix for JVM class names
    package_prefix: String,
    /// All resolved modules: file path → module info
    modules: HashMap<String, ResolvedModule>,
    /// Compilation order (topologically sorted)
    compile_order: Vec<String>,
}

impl ModuleGraph {
    pub fn new(package_prefix: &str) -> Self {
        Self {
            package_prefix: package_prefix.to_string(),
            modules: HashMap::new(),
            compile_order: Vec::new(),
        }
    }

    /// Register a module from a file path.
    /// Converts `src/services/math.ts` → `com/tsbyte/services/Math`
    pub fn register_module(&mut self, file_path: &str) -> String {
        let path = Path::new(file_path);
        let stem = path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Module".to_string());

        // Capitalize first letter for JVM convention
        let class_name = capitalize(&stem);

        // Build directory-based package path
        let parent = path.parent()
            .map(|p| p.to_string_lossy().replace('/', ".").replace('\\', "."))
            .unwrap_or_default();

        let jvm_name = if parent.is_empty() || parent == "." {
            format!("{}/{}", self.package_prefix.replace('.', "/"), class_name)
        } else {
            let clean_parent = parent.trim_start_matches("src.")
                .trim_start_matches("src")
                .trim_start_matches('.');
            if clean_parent.is_empty() {
                format!("{}/{}", self.package_prefix.replace('.', "/"), class_name)
            } else {
                format!("{}/{}/{}", self.package_prefix.replace('.', "/"), clean_parent.replace('.', "/"), class_name)
            }
        };

        let module = ResolvedModule {
            source_path: PathBuf::from(file_path),
            jvm_class_name: jvm_name.clone(),
            exports: HashMap::new(),
            imports: Vec::new(),
        };

        self.modules.insert(file_path.to_string(), module);
        self.compile_order.push(file_path.to_string());
        jvm_name
    }

    /// Add an exported symbol to a module
    pub fn add_export(&mut self, file_path: &str, name: &str, kind: ExportKind, descriptor: &str) {
        if let Some(module) = self.modules.get_mut(file_path) {
            module.exports.insert(name.to_string(), ExportedSymbol {
                name: name.to_string(),
                kind,
                descriptor: descriptor.to_string(),
            });
        }
    }

    /// Add an import reference to a module
    pub fn add_import(&mut self, file_path: &str, local_name: &str, source_module: &str, original_name: &str) {
        if let Some(module) = self.modules.get_mut(file_path) {
            module.imports.push(ImportRef {
                local_name: local_name.to_string(),
                source_module: source_module.to_string(),
                original_name: original_name.to_string(),
            });
        }
    }

    /// Resolve an import: given the importing file and the import specifier,
    /// return the target JVM class name and method descriptor.
    pub fn resolve_import(&self, from_file: &str, specifier: &str, symbol_name: &str) -> Option<(String, String)> {
        // Resolve relative path
        let from_dir = Path::new(from_file).parent().unwrap_or(Path::new("."));
        let raw_path = from_dir.join(specifier);
        let target_path = normalize_path(&raw_path);
        let target_with_ext = format!("{}.ts", target_path.to_string_lossy());

        // Look up in module registry
        let target_module = self.modules.get(&target_with_ext)
            .or_else(|| self.modules.get(&target_path.to_string_lossy().to_string()))?;

        let export = target_module.exports.get(symbol_name)?;

        Some((target_module.jvm_class_name.clone(), export.descriptor.clone()))
    }

    /// Get the JVM class name for a file
    pub fn get_class_name(&self, file_path: &str) -> Option<&str> {
        self.modules.get(file_path).map(|m| m.jvm_class_name.as_str())
    }

    /// Get all modules in compilation order
    pub fn modules_in_order(&self) -> Vec<&ResolvedModule> {
        self.compile_order.iter()
            .filter_map(|p| self.modules.get(p))
            .collect()
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(c) => {
                normalized.push(c);
            }
            _ => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registration() {
        let mut graph = ModuleGraph::new("com.tsbyte.app");
        let class = graph.register_module("src/services/math.ts");
        assert_eq!(class, "com/tsbyte/app/services/Math");
    }

    #[test]
    fn test_module_registration_root() {
        let mut graph = ModuleGraph::new("com.tsbyte.app");
        let class = graph.register_module("index.ts");
        assert_eq!(class, "com/tsbyte/app/Index");
    }

    #[test]
    fn test_export_and_resolve() {
        let mut graph = ModuleGraph::new("com.tsbyte.app");
        graph.register_module("src/utils.ts");
        graph.add_export("src/utils.ts", "add", ExportKind::Function, "(DD)D");

        graph.register_module("src/main.ts");
        let result = graph.resolve_import("src/main.ts", "./utils", "add");
        assert!(result.is_some());
        let (class, desc) = result.unwrap();
        assert_eq!(class, "com/tsbyte/app/Utils");
        assert_eq!(desc, "(DD)D");
    }

    #[test]
    fn test_parent_directory_resolve() {
        let mut graph = ModuleGraph::new("com.tsbyte.app");
        graph.register_module("src/utils/math.ts");
        graph.add_export("src/utils/math.ts", "multiply", ExportKind::Function, "(DD)D");

        graph.register_module("src/services/payment.ts");
        let result = graph.resolve_import("src/services/payment.ts", "../utils/math", "multiply");
        assert!(result.is_some(), "resolve_import returned None for parent directory specifier");
        let (class, desc) = result.unwrap();
        assert_eq!(class, "com/tsbyte/app/utils/Math");
        assert_eq!(desc, "(DD)D");
    }
}

