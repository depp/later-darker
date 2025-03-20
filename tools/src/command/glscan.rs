use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::Parser;

use crate::emit;
use crate::identifier;

/// Scan files for usage of OpenGL functions.
#[derive(Parser, Debug)]
pub struct Args {
    /// Source files to scan.
    sources: Vec<PathBuf>,

    /// Output file.
    #[arg(long)]
    output: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut entrypoints: HashSet<String> = HashSet::new();
        for file in self.sources.iter() {
            let file_entrypoints = read_entrypoints(file)?;
            entrypoints.extend(file_entrypoints);
        }
        let mut entrypoint_list: Vec<&str> = entrypoints.iter().map(|s| s.as_str()).collect();
        entrypoint_list.sort();
        let mut output = String::new();
        for &name in entrypoint_list.iter() {
            output.push_str(name);
            output.push('\n');
        }
        emit::write_or_stdout(self.output.as_deref(), output.as_bytes())?;
        Ok(())
    }
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
