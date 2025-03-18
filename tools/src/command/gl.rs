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
        for file in files.iter() {
            let text = fs::read_to_string(&file)?;
            eprintln!("File: {}", file.file_name().unwrap().to_str().unwrap());
            for ident in identifier::Identifiers::new(&text) {
                eprintln!("  {:?}", ident);
            }
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
