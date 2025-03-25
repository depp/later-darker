use crate::emit;
use crate::gl::scan;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

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
        let entrypoints = scan::read_entrypoints(&self.sources)?;
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
