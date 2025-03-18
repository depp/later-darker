use std::error::Error;
use std::fs;
use std::path::PathBuf;

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
        for item in fs::read_dir(&directory)? {
            let item = item?;
            let mut file_path = directory.clone();
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
            let text = fs::read_to_string(&file_path)?;
            eprintln!("File: {}", file_path.file_name().unwrap().to_str().unwrap());
            for ident in identifier::Identifiers::new(&text) {
                eprintln!("  {:?}", ident);
            }
        }
        Ok(())
    }
}
