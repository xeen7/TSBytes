use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use crate::error::{Result, TsDroidError};

pub struct JarPackager;

impl JarPackager {
    /// Packages compiled `.class` files into a valid, executable `.jar` archive.
    ///
    /// The archive layout will be:
    ///
    /// ```text
    /// META-INF/MANIFEST.MF
    /// com/example/App.class
    /// com/example/SomeOtherClass.class
    /// ```
    ///
    /// # Arguments
    /// * `classes` - Tuples of `(jvm_class_path, byte_array)` e.g. `("com/example/App", bytes)`.
    /// * `main_class` - Fully-qualified dotted class name for the manifest (e.g. `"com.example.App"`).
    /// * `output_path` - Destination path for the `.jar` file.
    pub fn package_jar(
        classes: &[(String, Vec<u8>)],
        main_class: &str,
        output_path: &Path,
    ) -> Result<()> {
        let file = File::create(output_path)
            .map_err(TsDroidError::Io)?;
        let mut zip = zip::ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // 1. Create META-INF directory and MANIFEST.MF
        zip.add_directory("META-INF/", options)
            .map_err(|e| TsDroidError::ZipError(e.to_string()))?;

        zip.start_file("META-INF/MANIFEST.MF", options)
            .map_err(|e| TsDroidError::ZipError(e.to_string()))?;

        // MANIFEST.MF spec: each header must be followed by \r\n, file ends with blank line
        write!(
            zip,
            "Manifest-Version: 1.0\r\nCreated-By: TSDroid Compiler\r\nMain-Class: {}\r\n\r\n",
            main_class
        )
        .map_err(TsDroidError::Io)?;

        // 2. Add all class files, creating package directories automatically
        let mut added_dirs = std::collections::HashSet::new();
        for (class_name, bytes) in classes {
            // Ensure slashes (JVM path format) — class names may come in dotted or slashed form.
            let normalized = class_name.replace('.', "/");

            // Add any missing intermediate directory entries for correctness
            if let Some(parent) = normalized.rfind('/') {
                let dir = format!("{}/", &normalized[..parent]);
                if added_dirs.insert(dir.clone()) {
                    zip.add_directory(&dir, options)
                        .map_err(|e| TsDroidError::ZipError(e.to_string()))?;
                }
            }

            let entry_name = format!("{}.class", normalized);
            zip.start_file(&entry_name, options)
                .map_err(|e| TsDroidError::ZipError(e.to_string()))?;
            zip.write_all(bytes)
                .map_err(TsDroidError::Io)?;
        }

        zip.finish()
            .map_err(|e| TsDroidError::ZipError(e.to_string()))?;

        Ok(())
    }
}
