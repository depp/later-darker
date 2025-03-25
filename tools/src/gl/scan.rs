use crate::error::FileError;
use crate::identifier;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Return true if this string matches the pattern expected for an OpenGL entry point.
fn is_entrypoint(identifier: &str) -> bool {
    identifier.len() >= 3
        && identifier.starts_with("gl")
        && identifier[2..].chars().next().unwrap().is_ascii_uppercase()
}

/// Get a set of all identifiers that look like OpenGL API entry points.
pub fn read_file_entrypoints(file_name: &Path) -> io::Result<HashSet<String>> {
    let text = fs::read_to_string(&file_name)?;
    let mut result = HashSet::new();
    for ident in identifier::Identifiers::new(&text) {
        if is_entrypoint(ident) {
            if !result.contains(ident) {
                result.insert(ident.to_string());
            }
        }
    }
    Ok(result)
}

/// Get all identifiers that look like OpenGL API entry points in the given files.
pub fn read_entrypoints(files: &[PathBuf]) -> Result<HashSet<String>, FileError> {
    let mut result = HashSet::new();
    for file in files.iter() {
        let file_entrypoints = match read_file_entrypoints(file) {
            Ok(value) => value,
            Err(err) => {
                return Err(FileError {
                    path: file.to_path_buf(),
                    error: err.into(),
                });
            }
        };
        result.extend(file_entrypoints);
    }
    Ok(result)
}
