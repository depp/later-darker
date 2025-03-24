use crate::emit;
use crate::gl;
use clap::Parser;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Generate OpenGL API bindings.
#[derive(Parser, Debug)]
pub struct Args {
    /// File with list of OpenGL functions, one per line.
    #[arg(long)]
    entry_points: Option<PathBuf>,

    /// Output C++ header file.
    #[arg(long)]
    output_header: Option<PathBuf>,

    /// Output C++ source file.
    #[arg(long)]
    output_data: Option<PathBuf>,
}

impl Args {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let api = gl::APISpec {
            version: gl::Version(3, 3),
            extensions: vec![],
        };
        let link = gl::APISpec {
            version: gl::Version(1, 1),
            extensions: vec![],
        };
        let api = gl::API::create(&api, &link)?;
        let bindings = match &self.entry_points {
            None => api.make_bindings(),
            Some(path) => {
                let text = fs::read_to_string(path)?;
                api.make_subset_bindings(text.lines())?
            }
        };
        emit::write_or_stdout(self.output_header.as_deref(), bindings.header.as_bytes())?;
        emit::write_or_stdout(self.output_data.as_deref(), bindings.data.as_bytes())?;
        Ok(())
    }
}
