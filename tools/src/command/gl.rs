use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Parser;

use crate::identifier;

#[derive(Parser, Debug)]
pub struct Args {
    srcdir: PathBuf,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut directory = self.srcdir.clone();
        directory.push("src");
        let files = find_cpp_files(&directory)?;
        let mut entrypoints: HashSet<String> = HashSet::new();
        for file in files.iter() {
            let file_entrypoints = read_entrypoints(file)?;
            entrypoints.extend(file_entrypoints);
        }
        let mut entrypoint_list: Vec<&str> = entrypoints.iter().map(|s| s.as_str()).collect();
        entrypoint_list.sort();
        eprintln!("Entry points:");
        for &name in entrypoint_list.iter() {
            eprintln!("  {}", name);
        }
        Ok(())
    }
}

/// List all C++ implementation files in the given directory.
fn find_cpp_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for item in fs::read_dir(&directory)? {
        let item = item?;
        let mut file_path = directory.to_path_buf();
        file_path.push(item.file_name());
        let Some(ext) = file_path.extension() else {
            continue;
        };
        let Some(ext) = ext.to_str() else {
            continue;
        };
        if ext != "cpp" {
            continue;
        }
        files.push(file_path);
    }
    Ok(files)
}

/// Get a set of all identifiers that look like OpenGL API entri
fn read_entrypoints(file_name: &Path) -> io::Result<HashSet<String>> {
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

/// Return true if this string matches the pattern expected for an OpenGL entry point.
fn is_entrypoint(identifier: &str) -> bool {
    identifier.len() >= 3
        && identifier.starts_with("gl")
        && identifier[2..].chars().next().unwrap().is_ascii_uppercase()
}
